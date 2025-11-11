use dishaster_views::Feedback;
use rand_distr::Normal;

use super::{feedback::*, ordering::*, prelude::*};

/// Queue of delayed serving communication messages to simulate human interaction latency.
#[derive(Resource, Default)]
pub struct ServingCommsQueue {
    pending: Vec<QueuedServingMessage>,
}

impl ServingCommsQueue {
    /// Schedule a message to be delivered at the specified simulation time.
    fn schedule(&mut self, deliver_at: f64, message: ServingMessage) {
        self.pending.push(QueuedServingMessage {
            deliver_at,
            message,
        });
    }

    /// Return every message that matured by the provided time.
    fn take_ready(&mut self, now: f64) -> impl Iterator<Item = ServingMessage> {
        self.pending
            .extract_if(.., move |item| item.deliver_at <= now)
            .map(|item| item.message)
    }
}

/// Metadata for a queued serving exchange.
struct QueuedServingMessage {
    deliver_at: f64,
    message: ServingMessage,
}

struct ServingMessage {
    diner: Entity,
    staff: Entity,
    kind: ServingMessageKind,
}

/// Distinct steps in the serving conversation.
enum ServingMessageKind {
    OrderSpoken,
    StaffConfirmed,
    DishReady,
}

/// Advance serving conversations using queued messages.
pub fn process_serving_messages(
    mut commands: Commands,
    mut sessions: Query<(Entity, &mut ServiceSession, &mut DinerState)>,
    mut staff_query: Query<(&ServingStaff, &mut ServingStaffState, &mut Movement)>,
    window_query: Query<&WindowDishes>,
    dish_query: Query<&Dish>,
    mut queue: ResMut<ServingCommsQueue>,
    time: Res<Time>,
    mut rng: ResMut<ServingRng>,
    mut feedback_messages: MessageWriter<FeedbackMessage>,
    registry: Res<GameModelRegistryRes>,
) {
    let now = time.current_time;
    // Deliver any elapsed step so the diner and staff state machines stay in sync.
    let ready = queue.take_ready(now).collect::<Vec<_>>();

    for ServingMessage { diner, staff, kind } in ready {
        let Ok((_, mut staff_state, _)) = staff_query.get_mut(staff) else {
            log::warn!(
                target: "serving",
                "Staff entity not found: {staff:?}"
            );
            continue;
        };
        let Ok((_, mut session, mut diner_state)) = sessions.get_mut(diner) else {
            log::warn!(
                target: "diner",
                "Diner session not found: {diner:?}"
            );
            // Diner is gone or session ended; free the staff slot.
            release_staff(&mut staff_state, now);
            continue;
        };
        let Some(request) = session.request.as_ref() else {
            // No active order means the chain already resolved; free staff.
            release_staff(&mut staff_state, now);
            continue;
        };

        match kind {
            ServingMessageKind::OrderSpoken => {
                if session.stage != ServiceStage::WaitingForStaffResponse
                    || session.staff != Some(staff)
                {
                    log::warn!(
                        target: "serving",
                        "Stale diner session: {diner:?} staff: {staff:?}"
                    );
                    // Ignore stale messages that belong to an older assignment.
                    continue;
                }

                log::debug!(
                    target: "serving",
                    "Staff entity found: {staff:?}"
                );
                staff_state.last_update_time = now;
                let feedback = Feedback::Thought(eco_format!("{}?", request.dish_name));
                feedback_messages.write(FeedbackMessage {
                    entity: staff,
                    content: feedback,
                    trigger: None,
                });
                let delay = rng.random_range(STAFF_CONFIRM_DELAY_MIN..STAFF_CONFIRM_DELAY_MAX);
                // Queue the verbal confirmation after a short pause to simulate speech.
                queue.schedule(
                    now + delay as f64,
                    ServingMessage {
                        diner,
                        staff,
                        kind: ServingMessageKind::StaffConfirmed,
                    },
                );
            }
            ServingMessageKind::StaffConfirmed => {
                if session.stage != ServiceStage::WaitingForStaffResponse
                    || session.staff != Some(staff)
                {
                    // Someone else took over; drop the duplicated reply.
                    continue;
                }

                // The staff member accepted the task, so we wait for food prep.
                staff_state.last_update_time = now;
                let confirm_feedback = Feedback::Thought(request.dish_name.clone());
                feedback_messages.write(FeedbackMessage {
                    entity: staff,
                    content: confirm_feedback,
                    trigger: None,
                });
                let diner_feedback = choose_feedback(&mut rng, feedbacks::DECIDING);
                feedback_messages.write(FeedbackMessage {
                    entity: diner,
                    content: diner_feedback,
                    trigger: None,
                });

                log::debug!(
                    target: "serving",
                    "Staff {} confirmed order for diner {}: {}",
                    staff,
                    diner,
                    request.dish_name
                );

                // Randomize prep time.
                let prep_time = {
                    let base = request.base_service_time;
                    let variation = rng
                        .random_range(-STAFF_SERVICE_TIME_VARIATION..STAFF_SERVICE_TIME_VARIATION);
                    base * (1.0 + variation)
                };
                // Schedule the ready notification to finish the conversation.
                queue.schedule(
                    now + prep_time as f64,
                    ServingMessage {
                        diner,
                        staff,
                        kind: ServingMessageKind::DishReady,
                    },
                );

                session.stage = ServiceStage::WaitingForDish; // Defer it to avoid borrow checker issues
            }
            ServingMessageKind::DishReady => {
                log::debug!(
                    target: "serving",
                    "Dish ready for diner {}",
                    diner,
                );

                if session.stage != ServiceStage::WaitingForDish || session.staff != Some(staff) {
                    log::warn!(
                        target: "serving",
                        "Inconsistent session state for diner {}",
                        diner,
                    );
                    // Another staffer already completed the order.
                    continue;
                }
                // Dish is ready - decide if ordering more after this
                session.stage = ServiceStage::DecideNextDish;

                let Some(request) = session.request.as_ref() else {
                    continue;
                };

                // Generate the served dish from current window state
                let service_time = (now - session.started_at) as f32;

                // Query window dishes to get current state and pricing
                if let Some(dish) = window_query.iter_descendants(session.window).find_map(|e| {
                    dish_query
                        .get(e)
                        .ok()
                        .filter(|d| d.model_id == request.dish_id)
                }) {
                    let Some(dish_model) = registry.dishes.get_by_id(&request.dish_id) else {
                        log::error!(
                            target: "diner",
                            "served dish id {:?} not found in registry!",
                            request.dish_id
                        );
                        continue;
                    };

                    // Sample actual portion weight from normal distribution
                    let served_weight = Normal::new(
                        dish_model.characteristics.weight_distrib.mean,
                        dish_model.characteristics.weight_distrib.stddev,
                    )
                    .unwrap()
                    .sample(&mut rng)
                    .max(0.01);

                    // Calculate price based on actual weight for ByWeight pricing
                    let price_paid = match dish.pricing {
                        PricingMethod::PerPortion(price) => price,
                        PricingMethod::ByWeight(price_per_kg) => {
                            // Use sampled weight to calculate actual price
                            price_per_kg * served_weight
                        }
                    };

                    // Create dish entity and add to served_dishes Vec
                    let dish_entity = commands
                        .spawn((
                            DisplayState {
                                proto: dish_model.display.res.clone(),
                                ..Default::default()
                            },
                            Transform {
                                ..Default::default()
                            },
                        ))
                        .id();

                    // Push to served_dishes Vec with actual weight
                    diner_state.served_dishes.push(ServedDish {
                        entity: dish_entity,
                        dish_id: request.dish_id.clone(),
                        served_weight,
                        remaining_weight: served_weight, // Initialize remaining weight
                        served_quality: dish.state.current_quality,
                        price_paid,
                        service_time,
                        contamination_level: dish.state.contamination_level,
                    });

                    // Update total spent
                    diner_state.total_spent += price_paid;
                } else {
                    log::warn!(
                        target: "serving",
                        "Could not find active dish for request {:?} in window {:?}",
                        request.dish_id,
                        session.window,
                    );
                }

                let staff_feedback = Feedback::Thought(eco_format!("{} ✅", request.dish_name));
                feedback_messages.write(FeedbackMessage {
                    entity: staff,
                    content: staff_feedback,
                    trigger: None,
                });
                let diner_feedback = choose_feedback(&mut rng, feedbacks::SERVING);
                feedback_messages.write(FeedbackMessage {
                    entity: diner,
                    content: diner_feedback,
                    trigger: None,
                });

                log::debug!(
                    target: "serving",
                    "Staff {} completed dish {} for diner {} ({}/{})",
                    staff,
                    request.dish_name,
                    diner,
                    session.current_dish_index + 1,
                    session.planned_order.len()
                );

                release_staff(&mut staff_state, now);
                session.staff = None;
            }
        }
    }
}

/// Reset the staff state so they re-enter the idle pool when conversations abort early
/// or when we detect stale messages.
fn release_staff(staff_state: &mut ServingStaffState, now: f64) {
    staff_state.reset(now);
}

fn staff_alignment_target(
    diner_pos: Vec2,
    staff: &ServingStaff,
    windows: &Query<&Window>,
) -> Option<Vec2> {
    let window = windows.get(staff.window).ok()?;
    Some(vec2(
        (diner_pos.x).clamp(window.location.x_min, window.location.x_max),
        window.location.y + WINDOW_STAFF_OFFSET,
    ))
}

/// Progress service sessions by allocating staff and queuing conversation beats.
pub fn drive_serving_sessions(
    mut diner_query: Query<
        (
            Entity,
            &mut ServiceSession,
            &mut DinerState,
            &Movement,
            &mut EntityRng,
            &DinerPersonality,
            &DinerPsychState,
            &DinerDiningProfile,
            &DinerLongTermMemory,
            &mut DinerShortTermMemory,
        ),
        With<Diner>,
    >,
    mut staff_query: Query<(&ServingStaff, &mut ServingStaffState, &mut Movement), Without<Diner>>,
    window_query: Query<&WindowDishes>,
    dish_query: Query<&Dish>,
    lane_query: Query<(&StaffForLane,)>,
    windows: Query<&Window>,
    registry: Res<GameModelRegistryRes>,
    mut comms: ResMut<ServingCommsQueue>,
    time: Res<Time>,
    mut feedback_messages: MessageWriter<FeedbackMessage>,
    ordering_config: Res<OrderingConfig>,
) {
    let now = time.current_time;

    for (
        diner,
        mut session,
        mut diner_state,
        diner_movement,
        mut rng,
        personality,
        psych_state,
        dining_profile,
        ltm,
        mut stm,
    ) in diner_query.iter_mut()
    {
        // Each phase of the service state machine either allocates resources,
        // waits on delayed conversation steps, or concludes the interaction.
        match session.stage {
            ServiceStage::AssignStaff => {
                let Ok((staff_for_lane,)) = lane_query.get(session.lane) else {
                    // Layout mismatch between queues and staff; wait for correction.
                    continue;
                };
                let staff_entity = staff_for_lane.staff;
                let Ok((staff, mut staff_state, mut staff_movement)) =
                    staff_query.get_mut(staff_entity)
                else {
                    // Staff entity might have despawned; let the next tick retry.
                    continue;
                };
                if !staff_state.is_idle() {
                    // Staff is still finishing another order; try again next frame.
                    continue;
                };

                // Decide the full order if not already planned
                if session.planned_order.is_empty() {
                    let Some(window_dishes) = window_query.get(session.window).ok() else {
                        // Diner is misconfigured without a window snapshot.
                        continue;
                    };

                    // Budget and spending are already initialized at spawn time
                    // Just reset the spending counter for this meal
                    diner_state.total_spent = 0.0;

                    // Call the ordering decision system to plan the full order
                    let planned_order = decide_order(
                        window_dishes,
                        &dish_query,
                        personality,
                        psych_state,
                        dining_profile,
                        ltm,
                        &mut stm,
                        &registry,
                        &ordering_config,
                        diner_state.meal_budget,
                        &mut rng,
                    );

                    if planned_order.is_empty() {
                        // No dishes available or no suitable dishes; complete session
                        staff_state.reset(now);
                        session.stage = ServiceStage::Completed;
                        continue;
                    }

                    session.planned_order = planned_order;
                    session.current_dish_index = 0;
                }

                // Get the next dish from the planned order
                if session.request.is_none() {
                    if session.current_dish_index >= session.planned_order.len() {
                        // All dishes have been processed, move to payment
                        session.stage = ServiceStage::Payment;
                        continue;
                    }

                    session.request =
                        Some(session.planned_order[session.current_dish_index].clone());
                }

                let dish_name = {
                    let request = session
                        .request
                        .as_ref()
                        .expect("service request must exist after initialization");
                    request.dish_name.clone()
                };
                session.staff = Some(staff_entity);
                session.stage = ServiceStage::WaitingForStaffResponse;
                staff_state.status = ServingStaffStatus::HandlingOrder;
                staff_state.current_session = Some(diner);
                staff_state.last_update_time = now;
                if let Some(target) = staff_alignment_target(diner_movement.pos, staff, &windows)
                    && !staff_movement.pos.close_to(target, 0.1)
                {
                    staff_movement.request_path(target);
                    log::debug!(
                        target: "staff",
                        "staff {} moving to align with diner {} at {:.2}",
                        staff_entity,
                        diner,
                        target
                    );
                }

                // Give the diner immediate feedback that their order was heard so they
                // perceive progress while we wait for the delayed response.
                feedback_messages.write(FeedbackMessage {
                    entity: diner,
                    content: Feedback::Thought(eco_format!("{}?", dish_name)),
                    trigger: None,
                });

                let delay: f32 = rng.random_range(ORDER_SPEECH_DELAY_MIN..ORDER_SPEECH_DELAY_MAX);
                comms.schedule(
                    now + f64::from(delay),
                    ServingMessage {
                        diner,
                        staff: staff_entity,
                        kind: ServingMessageKind::OrderSpoken,
                    },
                );
            }
            ServiceStage::WaitingForStaffResponse | ServiceStage::WaitingForDish => {
                // Keep the assigned staff aligned with the diner while the conversation plays out.
                if let Some(staff_entity) = session.staff
                    && let Ok((staff, _state, mut staff_movement)) =
                        staff_query.get_mut(staff_entity)
                    && let Some(target) =
                        staff_alignment_target(diner_movement.pos, staff, &windows)
                    && !staff_movement.pos.close_to(target, 0.1)
                    && staff_movement.pending_target.is_none()
                {
                    staff_movement.request_path(target);
                    log::debug!(
                        target: "staff",
                        "staff {} moving to align with diner {} at {:.2}",
                        staff_entity,
                        diner,
                        target
                    );
                }
            }
            ServiceStage::DecideNextDish => {
                // After receiving a dish, decide if ordering more
                session.current_dish_index += 1;

                if session.current_dish_index < session.planned_order.len() {
                    // More dishes to order - go back to AssignStaff to get next dish
                    session.stage = ServiceStage::AssignStaff;
                    session.request = None;
                } else {
                    // All dishes ordered - proceed to payment
                    session.stage = ServiceStage::Payment;
                }
            }
            ServiceStage::Payment => {
                // Payment/checkout stage - show total feedback and complete
                // TODO: Add payment animation/feedback here in future

                log::debug!(
                    target: "serving",
                    "diner {} completing payment, total dishes: {}",
                    diner,
                    session.planned_order.len()
                );

                // Transition to completed
                session.stage = ServiceStage::Completed;
            }
            ServiceStage::Completed => {}
        }
    }
}
