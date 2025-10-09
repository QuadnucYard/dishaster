use super::{feedback::*, prelude::*};

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

/// Distinct steps in the serving conversation.
enum ServingMessage {
    OrderSpoken { diner: Entity, staff: Entity },
    StaffConfirmed { diner: Entity, staff: Entity },
    DishReady { diner: Entity, staff: Entity },
}

/// Advance serving conversations using queued messages.
pub fn process_serving_messages(
    mut queue: ResMut<ServingCommsQueue>,
    time: Res<Time>,
    mut sessions: Query<(Entity, &mut ServiceSession)>,
    mut staff_query: Query<(&ServingStaff, &mut ServingStaffState, &mut Movement)>,
    mut events: ResMut<EventLog>,
    mut rng: ResMut<GameRng>,
) {
    let now = time.current_time;
    // Deliver any elapsed step so the diner and staff state machines stay in sync.
    let ready = queue.take_ready(now).collect::<Vec<_>>();

    for message in ready {
        match message {
            ServingMessage::OrderSpoken { diner, staff } => {
                let Ok((_, session)) = sessions.get_mut(diner) else {
                    // Diner is gone or session ended; free the staff slot.
                    release_staff(&mut staff_query, staff, now);
                    continue;
                };
                if session.stage != ServiceStage::WaitingForStaffResponse
                    || session.staff != Some(staff)
                {
                    // Ignore stale messages that belong to an older assignment.
                    continue;
                }
                let Some(request) = session.request.as_ref() else {
                    // No active order means the chain already resolved; free staff.
                    release_staff(&mut staff_query, staff, now);
                    continue;
                };
                // Confirm the staff member is still involved and surface the spoken order
                // back to the customer so both sides stay aligned.
                if let Ok((_, mut staff_state, _)) = staff_query.get_mut(staff) {
                    staff_state.last_update_time = now;
                    let feedback = Feedback::Thought(eco_format!("{}?", request.dish_name));
                    events.emit_feedback(FeedbackEvent {
                        entity: staff.into(),
                        content: feedback,
                        timestamp: now,
                    });
                    let delay = rng.random_range(STAFF_CONFIRM_DELAY_MIN..STAFF_CONFIRM_DELAY_MAX);
                    // Queue the verbal confirmation after a short pause to simulate speech.
                    queue.schedule(
                        now + f64::from(delay),
                        ServingMessage::StaffConfirmed { diner, staff },
                    );
                }
            }
            ServingMessage::StaffConfirmed { diner, staff } => {
                let Ok((_, mut session)) = sessions.get_mut(diner) else {
                    // Session vanished before confirmation landed.
                    release_staff(&mut staff_query, staff, now);
                    continue;
                };
                if session.stage != ServiceStage::WaitingForStaffResponse
                    || session.staff != Some(staff)
                {
                    // Someone else took over; drop the duplicated reply.
                    continue;
                }
                // The staff member accepted the task, so we wait for food prep.
                session.stage = ServiceStage::WaitingForDish;

                let Some(request) = session.request.as_ref() else {
                    // Bail if the order payload disappeared while confirming.
                    release_staff(&mut staff_query, staff, now);
                    continue;
                };
                let confirm_feedback = Feedback::Thought(eco_format!("{}", request.dish_name));
                let base = request.base_service_time.max(0.5);

                if let Ok((_, mut staff_state, _)) = staff_query.get_mut(staff) {
                    staff_state.last_update_time = now;
                    events.emit_feedback(FeedbackEvent {
                        entity: staff.into(),
                        content: confirm_feedback.clone(),
                        timestamp: now,
                    });
                    let diner_feedback =
                        Feedback::Thought(choose_feedback(&mut rng, DECIDING_FEEDBACKS).into());
                    events.emit_feedback(FeedbackEvent {
                        entity: diner.into(),
                        content: diner_feedback,
                        timestamp: now,
                    });

                    // Randomize prep time to avoid robotic behaviour each day.
                    let variation = rng
                        .random_range(-STAFF_SERVICE_TIME_VARIATION..STAFF_SERVICE_TIME_VARIATION);
                    let prep_time = (base * (1.0 + variation)).max(0.8);
                    // Schedule the ready notification to finish the conversation.
                    queue.schedule(
                        now + f64::from(prep_time),
                        ServingMessage::DishReady { diner, staff },
                    );
                }
            }
            ServingMessage::DishReady { diner, staff } => {
                let Ok((_, mut session)) = sessions.get_mut(diner) else {
                    // Diner left before the dish was ready; nothing to deliver.
                    release_staff(&mut staff_query, staff, now);
                    continue;
                };
                if session.stage != ServiceStage::WaitingForDish || session.staff != Some(staff) {
                    // Another staffer already completed the order.
                    continue;
                }
                // The loop closes once the dish is ready.
                session.stage = ServiceStage::Completed;

                let Some(request) = session.request.as_ref() else {
                    continue;
                };

                let staff_feedback = Feedback::Thought(eco_format!("{} ✅", request.dish_name));

                events.emit_feedback(FeedbackEvent {
                    entity: staff.into(),
                    content: staff_feedback,
                    timestamp: now,
                });
                let diner_feedback =
                    Feedback::Thought(choose_feedback(&mut rng, SERVING_FEEDBACKS).into());
                events.emit_feedback(FeedbackEvent {
                    entity: diner.into(),
                    content: diner_feedback,
                    timestamp: now,
                });

                release_staff(&mut staff_query, staff, now);
                session.staff = None;
            }
        }
    }
}

fn release_staff(
    query: &mut Query<(&ServingStaff, &mut ServingStaffState, &mut Movement)>,
    staff: Entity,
    now: f64,
) {
    // Reset the staff state so they re-enter the idle pool when conversations abort early
    // or when we detect stale messages.
    if let Ok((_staff_meta, mut state, mut movement)) = query.get_mut(staff) {
        state.reset(now);
        movement.stop();
    }
}

/// Progress service sessions by allocating staff and queuing conversation beats.
pub fn drive_serving_sessions(
    mut sessions: Query<(Entity, &mut ServiceSession, &Movement), With<Diner>>,
    staff_registry: Res<ServingStaffRegistry>,
    mut staff_query: Query<(&ServingStaff, &mut ServingStaffState, &mut Movement), Without<Diner>>,
    window_query: Query<&WindowDishes>,
    windows: Query<&Window>,
    registry: Res<GameModelRegistryRes>,
    mut comms: ResMut<ServingCommsQueue>,
    mut rng: ResMut<GameRng>,
    time: Res<Time>,
    mut events: ResMut<EventLog>,
    canteen: Res<Canteen>,
) {
    let now = time.current_time;

    for (diner, mut session, diner_movement) in sessions.iter_mut() {
        // Each phase of the service state machine either allocates resources,
        // waits on delayed conversation steps, or concludes the interaction.
        match session.stage {
            ServiceStage::AssignStaff => {
                let Some(staff_candidates) = staff_registry.staff_for(session.window) else {
                    // Window has no configured staff; keep waiting.
                    continue;
                };
                let Some(lane_index) = session.lane_index else {
                    // Lane not yet known; retry after queue assignment stabilizes.
                    continue;
                };
                let Some(&staff_entity) = staff_candidates.get(lane_index) else {
                    // Layout mismatch between queues and staff; wait for correction.
                    continue;
                };
                let Ok((staff_meta, mut staff_state, mut staff_movement)) =
                    staff_query.get_mut(staff_entity)
                else {
                    // Staff entity might have despawned; let the next tick retry.
                    continue;
                };
                if !staff_state.is_idle() {
                    // Staff is still finishing another order; try again next frame.
                    continue;
                };

                if session.request.is_none() {
                    let Some(window_dishes) = window_query.get(session.window).ok() else {
                        // Diner is misconfigured without a window snapshot.
                        continue;
                    };
                    let Some(request) = choose_service_request(window_dishes, &registry, &mut rng)
                    else {
                        // No dishes left in the window; mark the session complete gracefully.
                        staff_state.reset(now);
                        session.stage = ServiceStage::Completed;
                        continue;
                    };
                    session.request = Some(request);
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
                if let Some(target) =
                    staff_alignment_target(diner_movement.pos, staff_meta, &windows, &canteen)
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
                events.emit_feedback(FeedbackEvent {
                    entity: diner.into(),
                    content: Feedback::Thought(eco_format!("{}?", dish_name)),
                    timestamp: now,
                });

                let delay = rng.random_range(ORDER_SPEECH_DELAY_MIN..ORDER_SPEECH_DELAY_MAX);
                comms.schedule(
                    now + f64::from(delay),
                    ServingMessage::OrderSpoken {
                        diner,
                        staff: staff_entity,
                    },
                );
            }
            ServiceStage::WaitingForStaffResponse | ServiceStage::WaitingForDish => {
                // Keep the assigned staff aligned with the diner while the conversation plays out.
                if let Some(staff_entity) = session.staff
                    && let Ok((staff_meta, _state, mut staff_movement)) =
                        staff_query.get_mut(staff_entity)
                    && let Some(target) =
                        staff_alignment_target(diner_movement.pos, staff_meta, &windows, &canteen)
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
            ServiceStage::Completed => {}
        }
    }
}

fn choose_service_request(
    dishes: &WindowDishes,
    registry: &GameModelRegistry,
    rng: &mut GameRng,
) -> Option<ServiceRequest> {
    if dishes.dishes.is_empty() {
        return None;
    }

    // Pick a dish that is currently staged in the serving window.
    let active = dishes.dishes.choose(rng).unwrap();
    let dish_handle = registry
        .dishes
        .get_handle_by_id(&active.assignment.dish_id)?;
    // Look up the dish model to copy presentation details for feedback later.
    let dish_model = registry.dishes.get(dish_handle);

    // Populate the request struct that sessions carry for the rest of the workflow.
    Some(ServiceRequest {
        dish_id: active.assignment.dish_id.clone(),
        dish_slot: active.assignment.slot_index,
        dish_name: dish_model.name.clone(),
        base_service_time: dish_model.characteristics.serving_time,
    })
}

fn staff_alignment_target(
    diner_pos: Vec2,
    staff: &ServingStaff,
    windows: &Query<&Window>,
    canteen: &Canteen,
) -> Option<Vec2> {
    let window = windows.get(staff.window).ok()?;
    let y = (canteen.model.windows_y + WINDOW_STAFF_OFFSET).clamp(0.0, canteen.model.height);
    let x = diner_pos
        .x
        .clamp(window.location.x_min, window.location.x_max);
    Some(Vec2::new(x, y))
}
