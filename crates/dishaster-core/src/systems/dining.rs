//! Diner behavior and state machine system.

use bevy_ecs::schedule::ScheduleConfigs;
use dishaster_navigation::NavigationGrid;
use dishaster_views::{Feedback, FeedbackView};
use ordered_float::NotNan;

use super::{decision::*, feedback::*, prelude::*};

/// Collection of schedules dining systems
pub fn dining_systems() -> ScheduleConfigs<Box<dyn System<In = (), Out = ()> + 'static>> {
    (
        update_diner_goals,
        update_diner_psychology,
        (
            handle_enter_goal,
            handle_observe_goal,
            handle_decide_window_goal,
            handle_pick_tray_goal,
            handle_pick_chopsticks_goal,
            check_queue_patience,
            handle_queue_for_window_goal,
            handle_get_served_goal,
            handle_find_seat_goal,
            handle_move_to_seat_goal,
            handle_eat_goal,
            handle_return_dishes_goal,
            handle_leave_goal,
        )
            .chain(),
    )
        .into_configs()
}

fn update_diner_goals(diner_query: Query<(&mut DinerGoalState,)>, time: Res<Time>) {
    let dt = time.tick_duration as f32;
    for (mut goal,) in diner_query {
        goal.step(dt);
    }
}

/// Update psychological states (mood decay, patience adjustment)
fn update_diner_psychology(
    mut diner_query: Query<(&DinerPersonality, &mut DinerPsychState)>,
    time: Res<Time>,
) {
    const TAU_MOOD: f32 = 600.0; // Mood decays with 600 second time constant

    let dt = time.tick_duration as f32;

    for (personality, mut psych_state) in diner_query.iter_mut() {
        // Apply mood decay toward baseline
        apply_mood_decay(&mut psych_state, dt, TAU_MOOD);

        // Update patience based on current mood and trust
        update_patience(personality, &mut psych_state);
    }
}

fn handle_enter_goal(
    diner_query: Query<(&mut DinerGoalState, &mut Movement, &mut EntityRng)>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
) {
    for (mut goal, mut movement, mut rng) in diner_query {
        if !goal.is(DinerGoal::Enter) {
            continue;
        }

        if movement.target_reached && rng.random_bool(0.5) {
            goal.update(DinerGoal::Observe);
        } else if !movement.has_path() {
            // Spawn already sets pos; here we ensure the first wander target is reasonable.
            let spot = find_valid_spot_near(
                movement.pos + Vec2::Y * rng.random_range(1.0..3.0),
                3.0,
                &nav_grid,
                &mut rng,
            );
            movement.request_path(spot);
            log::debug!(
                target: "diner",
                "entering: pos={:.2} target={:.2}",
                movement.pos,
                spot
            );
        }
    }
}

fn handle_observe_goal(
    diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerTargets,
        &mut Movement,
        &DinerPersonality,
        &mut EntityRng,
    )>,
    window_query: Query<(Entity, &Window)>,
    time: Res<Time>,
    mut events: ResMut<EventQueue>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
) {
    for (entity, mut goal, mut targets, mut movement, personality, mut rng) in diner_query {
        if !goal.is(DinerGoal::Observe) {
            continue;
        }

        // If no window is being observed, or if we've been observing for too long, pick a new one.
        if targets.observing_window.is_none()
            || !movement.has_path() && goal.timer > 5.0 / personality.decisiveness.max(0.1)
        {
            // Simple logic: pick a random available window to observe.
            let available_windows = window_query.iter().filter(|(_, w)| w.enabled);

            let Some((window_entity, window)) = available_windows.choose(&mut rng) else {
                // No windows available, decide to leave.
                goal.update(DinerGoal::Leave);
                continue;
            };

            targets.observing_window = Some(window_entity);
            // Find a valid observation spot near the window.
            let observation_center = window.location.center()
                + vec2(rng.random_range(-1.0..1.0), rng.random_range(-3.0..-1.0));
            let target_pos = find_valid_spot_near(observation_center, 1.5, &nav_grid, &mut rng);

            movement.request_path(target_pos);
            log::debug!(
                target: "diner",
                "observing: window={window_entity:?} target={target_pos:.2})",
            );

            goal.reset_timer();
        }

        // If the diner has reached their observation spot, transition to deciding.
        if movement.target_reached {
            goal.update(DinerGoal::DecideWindow);
            continue;
        }

        if rng.random_bool(0.01) {
            events.emit_feedback(FeedbackView {
                entity: entity.to_entity_id(),
                content: Feedback::Thought(choose_feedback(&mut rng, OBSERVING_FEEDBACKS).into()),
                timestamp: time.current_time,
            })
        }
    }
}

fn handle_decide_window_goal(
    diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerTargets,
        &DinerPersonality,
        &mut DinerPsychState,
        &DinerLongTermMemory,
        &mut EntityRng,
    )>,
    window_query: Query<(Entity, &Window, &WindowDishes)>,
    lane_query: Query<(&QueueLane, &QueueLaneMembers)>,
    registry: Res<GameModelRegistryRes>,
    time: Res<Time>,
    mut events: ResMut<EventQueue>,
) {
    let config = DecisionConfig::default();

    for (entity, mut goal, mut targets, personality, mut psych_state, ltm, mut rng) in diner_query {
        if !goal.is(DinerGoal::DecideWindow) {
            continue;
        }

        // Check decision_time
        // todo: this should be probabilistic
        if goal.timer < 3.0 / personality.decisiveness.max(0.1) {
            continue;
        }

        // Update patience based on current psychological state
        update_patience(personality, &mut psych_state);

        // Gather all visible windows and evaluate them
        let mut candidates = Vec::new();

        for (window_entity, window, window_dishes) in window_query.iter() {
            if !window.enabled {
                continue;
            }

            // Get queue length for this window by finding its lane
            let queue_length = lane_query
                .iter()
                .find(|(lane, _)| lane.owner == window_entity)
                .map(|(_, members)| members.members.len())
                .unwrap_or(0);

            // Estimate service time (placeholder - should come from window model)
            let avg_service_time = 10.0; // seconds per person

            if let Some(candidate) = evaluate_window(
                window_entity,
                &window_dishes.dishes,
                queue_length,
                avg_service_time,
                personality,
                &psych_state,
                ltm,
                &registry,
                &config,
            ) {
                candidates.push(candidate);
            }
        }

        // Select window using softmax sampling
        let chosen = select_window_from_candidates(&candidates, &config, &mut rng);

        let Some(window_entity) = chosen else {
            // No suitable window found, go back to observing
            log::debug!(target: "diner", "no suitable window found, continue observing");
            goal.update(DinerGoal::Observe);
            continue;
        };

        log::info!(target: "diner", "decision: choose_window entity={window_entity:?}");

        // Give feedback
        if rng.random_bool(0.5) {
            events.emit_feedback(FeedbackView {
                entity: entity.to_entity_id(),
                content: Feedback::Thought(choose_feedback(&mut rng, DECIDING_FEEDBACKS).into()),
                timestamp: time.current_time,
            });
        }

        targets.chosen_window = Some(window_entity);

        goal.update(DinerGoal::PickTray);
    }
}

fn handle_pick_tray_goal(
    mut commands: Commands,
    diner_query: Query<(
        Entity,
        &mut DinerState,
        &mut DinerGoalState,
        &mut DinerTargets,
        &mut Movement,
        &mut EntityRng,
    )>,
    mut dispenser_query: Query<(Entity, &mut Dispenser)>,
    registry: Res<GameModelRegistryRes>,
    mut events: ResMut<EventQueue>,
) {
    for (entity, mut state, mut goal, mut targets, mut movement, mut rng) in diner_query {
        if !goal.is(DinerGoal::PickTray) {
            continue;
        }

        if targets.tray_target.is_none() {
            // Choose the closest tray dispenser with stock
            let Some((dispenser_entity, dispenser)) = dispenser_query
                .iter()
                .filter(|(_, d)| {
                    d.dispenser_type == DispenserType::Tray
                        && (!d.center_pos.close_to(movement.pos, 3.0) || d.current_stock > 0)
                })
                .min_by_key(|(_, d)| {
                    let distance = movement.pos.distance_squared(d.center_pos);
                    NotNan::new(distance).unwrap()
                })
            else {
                // No dispensers available
                log::warn!(
                    target: "diner",
                    "no_tray_dispenser: entity={entity:?}"
                );
                continue;
            };
            targets.tray_target = Some(dispenser_entity);
            log::debug!(
                target: "diner",
                "picking_tray: target={dispenser_entity:?}"
            );
            movement.request_path_to_rect(dispenser.reception_area);
            continue;
        }

        if !movement.target_reached {
            continue;
        }

        // Reached the dispenser - check stock before taking
        let Some(tray_dispenser_entity) = targets.tray_target else {
            continue;
        };
        let Ok((_, mut dispenser)) = dispenser_query.get_mut(tray_dispenser_entity) else {
            continue;
        };

        // Check if dispenser still has stock
        if dispenser.current_stock == 0 {
            log::warn!(
                target: "diner",
                "tray_dispenser_empty: entity={entity:?}, dispenser={tray_dispenser_entity:?}"
            );
            // Dispenser is empty, find another one
            targets.tray_target = None;
            continue;
        }

        // Deduct stock
        dispenser.current_stock = dispenser.current_stock.saturating_sub(1);

        let dispenser_model = registry.dispensers.get(dispenser.model);

        log::debug!(
            target: "diner",
            "picked_tray: entity={entity:?}, remaining_stock={}",
            dispenser.current_stock
        );

        // Emit stock changed event
        events.push(SimEvent::DispenserStockChanged {
            entity: tray_dispenser_entity.to_entity_id(),
            current_stock: dispenser.current_stock,
            capacity: dispenser_model.capacity,
        });

        // Spawn the tray item
        let tray_res = dispenser_model.item_display.res.clone();
        let tray_entity = commands
            .spawn((
                DisplayState {
                    proto: tray_res,
                    ..Default::default()
                },
                Transform {
                    ..Default::default()
                },
            ))
            .id();

        state.tray = Some(tray_entity);

        events.push(SimEvent::DinerItemsChanged {
            entity: entity.to_entity_id(),
            change: DinerItemsChange::PickTray(tray_entity.to_entity_id()),
        });

        goal.update(if rng.random_bool(0.3) {
            // 30% chance to pick chopsticks next
            DinerGoal::PickChopsticks
        } else {
            DinerGoal::QueueForWindow
        });
    }
}

fn handle_pick_chopsticks_goal(
    mut commands: Commands,
    diner_query: Query<(
        Entity,
        &mut DinerState,
        &mut DinerGoalState,
        &mut DinerTargets,
        &mut Movement,
    )>,
    mut dispenser_query: Query<(Entity, &mut Dispenser)>,
    registry: Res<GameModelRegistryRes>,
    mut events: ResMut<EventQueue>,
) {
    for (entity, mut state, mut goal, mut targets, mut movement) in diner_query {
        if !goal.is(DinerGoal::PickChopsticks) {
            continue;
        }

        if targets.chopstick_target.is_none() {
            // Choose the closest chopstick dispenser with stock
            let Some((dispenser_entity, dispenser)) = dispenser_query
                .iter()
                .filter(|(_, d)| {
                    d.dispenser_type == DispenserType::Chopstick
                        && (!d.center_pos.close_to(movement.pos, 3.0) || d.current_stock > 0)
                })
                .min_by_key(|(_, d)| {
                    let distance = movement.pos.distance_squared(d.center_pos);
                    NotNan::new(distance).unwrap()
                })
            else {
                // No dispensers available
                log::warn!(
                    target: "diner",
                    "no_chopstick_dispenser: entity={entity:?}"
                );
                continue;
            };
            targets.chopstick_target = Some(dispenser_entity);
            log::debug!(
                target: "diner",
                "pick_chopsticks_target: target={dispenser_entity:?}, stock={}",
                dispenser.current_stock
            );
            movement.request_path_to_rect(dispenser.reception_area);
            continue;
        }

        if !movement.target_reached {
            continue;
        }

        // Reached the dispenser - check stock before taking
        let Some(chopstick_dispenser_entity) = targets.chopstick_target else {
            continue;
        };

        let Ok((_, mut dispenser)) = dispenser_query.get_mut(chopstick_dispenser_entity) else {
            continue;
        };

        // Check if dispenser still has stock
        if dispenser.current_stock == 0 {
            log::warn!(
                target: "diner",
                "chopstick_dispenser_empty: entity={entity:?}, dispenser={chopstick_dispenser_entity:?}"
            );
            // Dispenser is empty, find another one
            targets.chopstick_target = None;
            continue;
        }

        // Deduct stock
        dispenser.current_stock = dispenser.current_stock.saturating_sub(1);

        let dispenser_model = registry.dispensers.get(dispenser.model);

        log::debug!(
            target: "diner",
            "picked_chopsticks: entity={entity:?}, remaining_stock={}",
            dispenser.current_stock
        );

        // Emit stock changed event
        events.push(SimEvent::DispenserStockChanged {
            entity: chopstick_dispenser_entity.to_entity_id(),
            current_stock: dispenser.current_stock,
            capacity: dispenser_model.capacity,
        });

        // Spawn the chopsticks item
        let chopsticks_res = dispenser_model.item_display.res.clone();
        let chopsticks_entity = commands
            .spawn((
                DisplayState {
                    proto: chopsticks_res,
                    ..Default::default()
                },
                Transform {
                    ..Default::default()
                },
            ))
            .id();

        state.chopsticks = Some(chopsticks_entity);

        events.push(SimEvent::DinerItemsChanged {
            entity: entity.to_entity_id(),
            change: DinerItemsChange::PickChopsticks(chopsticks_entity.to_entity_id()),
        });

        goal.update(if state.served_dish.is_some() {
            DinerGoal::FindSeat
        } else {
            println!("Chopsticks picked before being served!");
            DinerGoal::QueueForWindow
        });
    }
}

/// Check if diners in queue have run out of patience and should abandon
fn check_queue_patience(
    mut commands: Commands,
    mut diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerPsychState,
        &mut DinerLongTermMemory,
        &QueueMember,
    )>,
    lane_query: Query<&QueueLaneMembers>,
) {
    let config = DecisionConfig::default();

    for (entity, mut goal, mut psych_state, mut ltm, queue_member) in diner_query.iter_mut() {
        if !goal.is(DinerGoal::QueueForWindow) {
            continue;
        }

        // Estimate wait time based on queue position
        let queue_length = lane_query
            .get(queue_member.lane)
            .map(|members| members.members.len())
            .unwrap_or(1);

        let estimated_wait = queue_length as f32 * 10.0; // Rough estimate: 10s per person
        let patience_now = psych_state.patience;

        // Check if patience exceeded
        if estimated_wait > patience_now {
            log::info!(
                target: "diner",
                "diner {:?} abandoning queue due to patience (wait={:.1}s, patience={:.1}s)",
                entity,
                estimated_wait,
                patience_now
            );

            // Apply abandonment penalty
            handle_abandon_penalty(
                &mut psych_state,
                &mut ltm,
                estimated_wait,
                patience_now,
                &config,
            );

            // Leave queue and exit canteen
            commands.entity(entity).remove::<QueueMember>();
            goal.update(DinerGoal::Leave);
        }
    }
}

fn handle_queue_for_window_goal(
    mut commands: Commands,
    diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &DinerTargets,
        Option<&QueueIntent>,
        Option<&QueueMember>,
    )>,
    window_query: Query<&LaneOwner, With<Window>>,
    lane_query: Query<(&QueueLane, &QueueLaneMembers)>,
    time: Res<Time>,
) {
    for (entity, mut goal, targets, queue_intent, queue_member) in diner_query {
        if !goal.is(DinerGoal::QueueForWindow) {
            continue;
        }

        if queue_intent.is_none() && queue_member.is_none() {
            // Not yet queued, choose a lane to queue for
            let Some(window_entity) = targets.chosen_window else {
                // No chosen window, go back to deciding
                goal.update(DinerGoal::DecideWindow);
                continue;
            };

            // Choose a lane with the shortest queue of that window
            let lane_entity = window_query
                .get(window_entity)
                .expect("window should exist")
                .lanes
                .iter()
                .map(|&lane_entity| (lane_entity, lane_query.get(lane_entity).unwrap().1))
                .min_by_key(|(_, members)| members.members.len())
                .map(|(lane_entity, _)| lane_entity)
                .expect("window should have at least one lane");

            commands
                .entity(entity)
                .insert(QueueIntent::new(lane_entity)); // intent to queue

            continue;
        }

        if let Some(queue_member) = queue_member
            && queue_member.ranking == 0
            && let Ok((lane, _)) = lane_query.get(queue_member.lane)
        {
            goal.update(DinerGoal::GetServed);
            commands.entity(entity).insert(ServiceSession::new(
                lane.owner,
                queue_member.lane,
                time.current_time,
            ));
            continue;
        }
    }
}

fn handle_get_served_goal(
    mut commands: Commands,
    diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &DinerState,
        &DinerTargets,
        &ServiceSession,
    )>,
    mut events: ResMut<EventQueue>,
) {
    for (entity, mut goal, state, targets, session) in diner_query {
        if !goal.is(DinerGoal::GetServed) {
            continue;
        }

        if session.stage != ServiceStage::Completed {
            continue;
        }

        log::debug!(
            target: "diner",
            "diner {} finished being served, moving to find seat",
            entity
        );

        // Service completed, remove session and move to finding a seat
        commands
            .entity(entity)
            .remove::<QueueMember>()
            .remove::<ServiceSession>();

        let Some(served_dish) = &state.served_dish else {
            log::error!(
                target: "diner",
                "diner {} has no served dish after service completion!",
                entity
            );
            continue;
        };
        events.push(SimEvent::DinerItemsChanged {
            entity: entity.to_entity_id(),
            change: DinerItemsChange::PickDish(served_dish.entity.to_entity_id()),
        });

        goal.update(if targets.chopstick_target.is_some() {
            DinerGoal::FindSeat
        } else {
            DinerGoal::PickChopsticks // need chopsticks
        });
    }
}

fn handle_find_seat_goal(
    diner_query: Query<(
        &mut DinerGoalState,
        &mut DinerTargets,
        &mut Movement,
        &mut EntityRng,
    )>,
    table_query: Query<(Entity, &mut DiningTable)>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
) {
    for (mut goal, mut targets, mut movement, mut rng) in diner_query {
        if !goal.is(DinerGoal::FindSeat) {
            continue;
        }

        if !(goal.timer > 3.0 && rng.random_bool(0.1)) {
            continue;
        }

        if let Some((table_entity, seat_index)) = find_seat(movement.pos, &table_query, &mut rng) {
            let table = table_query.get(table_entity).unwrap().1;
            targets.chosen_seat = Some((table_entity, seat_index));
            let seat_pos = table.seat_positions[seat_index];
            log::debug!(
                target: "diner",
                "found_seat: table={table_entity:?} seat={seat_index} target={seat_pos:.2}",
            );
            goal.update(DinerGoal::MoveToSeat);
            // The navigation is left for MoveToSeat handler
        } else {
            log::debug!(
                target: "diner",
                "find_seat: no seats available, wandering"
            );
            // Retry later. Wonder around a bit.
            goal.reset_timer();
            if let Some(spot) =
                try_find_valid_spot_near(movement.pos, WANDER_RADIUS, &nav_grid, &mut rng)
            {
                movement.request_path(spot);
                log::debug!(
                    target: "diner",
                    "find_seat: wandering to {:.2}",
                    spot
                );
            }
        }
    }
}

fn find_seat(
    pos: Vec2,
    table_query: &Query<(Entity, &mut DiningTable)>,
    rng: &mut EntityRng,
) -> Option<(Entity, usize)> {
    // scoring factors (lower is better):
    // - distance
    // - dirtiness
    // - occupancy

    let (table_entity, table) = table_query
        .iter()
        .filter(|(_, table)| table.occupants.iter().any(|o| o.is_none()))
        .min_by_key(|(_, table)| {
            let distance = pos.distance_squared(table.center_pos);
            let dirtiness = table.dirtiness;
            let occupancy = table.occupants.iter().filter(|o| o.is_some()).count();
            let score = distance * (dirtiness + 0.5) * ((occupancy as f32).squared() + 1.0);
            NotNan::new(score).unwrap()
        })?;

    // Randomly pick one of the free seats at the chosen table
    let available_seats: Vec<_> = table
        .occupants
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_none())
        .map(|(i, _)| i)
        .collect();
    available_seats
        .choose(rng)
        .map(|&seat_index| (table_entity, seat_index))
}

fn handle_move_to_seat_goal(
    diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerTargets,
        &mut Movement,
    )>,
    mut table_query: Query<(Entity, &mut DiningTable)>,
    mut events: ResMut<EventQueue>,
) {
    for (entity, mut goal, mut targets, mut movement) in diner_query {
        if !goal.is(DinerGoal::MoveToSeat) {
            continue;
        }

        let Some((table_entity, seat_index)) = targets.chosen_seat else {
            log::debug!(
                target: "diner",
                "move_to_seat: no seat chosen, going back to finding"
            );
            // No seat chosen any more, go back to finding
            goal.update(DinerGoal::FindSeat);
            continue;
        };

        let mut table = table_query
            .get_mut(table_entity)
            .expect("table should exist")
            .1;

        // Check if the chosen seat is still valid
        if table.occupants[seat_index].is_some() {
            log::debug!(
                target: "diner",
                "move_to_seat: seat taken, need to find another"
            );
            // Seat taken, need to find another
            targets.chosen_seat = None;
            goal.update(DinerGoal::FindSeat);
            continue;
        }

        // Check reaching the seat
        let seat_pos = table.seat_positions[seat_index];
        if movement.pos.close_to(seat_pos, TABLE_SEAT_ARRIVAL_EPS) {
            log::debug!(
                target: "diner",
                "seated: table={table_entity:?} seat={seat_index} pos={:.2}",
                seat_pos
            );
            // Reached the seat
            movement.stop_as_reached();
            movement.pos = seat_pos; // snap to seat position
            table.occupants[seat_index] = Some(entity); // Mark the seat as occupied

            events.push(SimEvent::DinerItemsChanged {
                entity: entity.to_entity_id(),
                change: DinerItemsChange::StartEating,
            });

            goal.update(DinerGoal::Eat);
            continue;
        }

        if !movement.has_path() {
            // Somehow lost path, re-request
            movement.request_path(seat_pos);
            log::debug!(
                target: "diner",
                "move_to_seat: diner={entity:?} re-requesting path to table={table_entity:?} seat={seat_index} target={:.2}",
                seat_pos
            );
        }
    }
}

fn handle_eat_goal(
    mut diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerState,
        &mut DinerTargets,
        &DinerDiningProfile,
        &mut DinerPsychState,
        &mut DinerLongTermMemory,
        &mut EntityRng,
    )>,
    mut table_query: Query<(Entity, &mut DiningTable)>,
    registry: Res<GameModelRegistryRes>,
    time: Res<Time>,
    mut events: ResMut<EventQueue>,
) {
    let satisfaction_weights = SatisfactionWeights::default();

    for (entity, mut goal, state, mut targets, dining_profile, mut psych_state, mut ltm, mut rng) in
        diner_query.iter_mut()
    {
        if !goal.is(DinerGoal::Eat) {
            continue;
        }

        // Calculate eating time based on diner's eating speed
        // Default base eating time is 30 seconds, modified by eating speed
        const DEFAULT_EATING_TIME: f32 = 30.0;
        let eating_time = DEFAULT_EATING_TIME / dining_profile.eating_speed;

        if goal.timer < eating_time {
            continue;
        }

        // Finished eating - update memory and psychological state
        if let Some(ref served_dish) = state.served_dish {
            // Get dish model for tags and base price
            let dish_tags = registry
                .dishes
                .get_by_id(&served_dish.dish_id)
                .map(|m| m.characteristics.tags.as_slice())
                .unwrap_or(&[]);

            // Use base price from model if available
            let base_price = registry
                .dishes
                .get_by_id(&served_dish.dish_id)
                .and_then(|m| {
                    if m.characteristics.base_price > 0.0 {
                        Some(m.characteristics.base_price)
                    } else {
                        None
                    }
                })
                .unwrap_or(served_dish.price_paid * 0.9);

            update_after_eating(
                dish_tags,
                &served_dish.dish_id,
                served_dish.price_paid,
                base_price,
                served_dish.served_quality,
                served_dish.contamination_level,
                time.current_time as f32,
                &mut psych_state,
                &mut ltm,
                &satisfaction_weights,
            );
        }

        // Free the table seat
        let (table_entity, seat_index) = targets.chosen_seat.expect("should have chosen seat");
        let mut table = table_query
            .get_mut(table_entity)
            .expect("table should exist")
            .1;
        table.dirtiness += rng.random_range(0.01..0.2); // increase dirtiness. todo: this should be decided by dish and diner
        table.occupants[seat_index] = None; // Free the seat
        targets.chosen_seat = None;

        log::debug!(
            target: "diner",
            "finished_eating: table={table_entity:?} seat={seat_index} pos={:.2}",
            table.seat_positions[seat_index]
        );

        events.push(SimEvent::DinerItemsChanged {
            entity: entity.to_entity_id(),
            change: DinerItemsChange::FinishEating,
        });

        goal.update(DinerGoal::ReturnDishes);
    }
}

fn handle_return_dishes_goal(
    mut commands: Commands,
    diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerState,
        &mut DinerTargets,
        &mut Movement,
    )>,
    collector_query: Query<(Entity, &DishCollector)>,
    mut events: ResMut<EventQueue>,
) {
    if collector_query.is_empty() {
        return;
    }

    for (entity, mut goal, mut state, mut targets, mut movement) in diner_query {
        if !goal.is(DinerGoal::ReturnDishes) {
            continue;
        }

        if targets.collector_target.is_none() {
            let best = collector_query.iter().min_by_key(|(_, collector)| {
                let distance = movement.pos.distance_squared(collector.center_pos);
                NotNan::new(distance).unwrap()
            });

            let Some((collector_entity, collector)) = best else {
                // No collectors available, just leave
                goal.update(DinerGoal::Leave);
                continue;
            };

            targets.collector_target = Some(collector_entity);
            movement.request_path_to_rect(collector.reception_area);
            log::debug!(
                target: "diner",
                "returning_dishes: target={collector_entity:?} pos={:.2}",
                collector.center_pos
            );
            continue;
        }

        if movement.target_reached {
            movement.stop_as_reached();
            if goal.timer < 2.0 {
                // Simulate time taken to return dishes
                continue;
            }
            log::debug!(
                target: "diner",
                "dishes_returned: pos={:.2}",
                movement.pos
            );

            // Despawn tablewares
            if let Some(chopsticks_entity) = state.chopsticks.take() {
                commands.entity(chopsticks_entity).despawn();
            }
            if let Some(tray_entity) = state.tray.take() {
                commands.entity(tray_entity).despawn();
            }
            if let Some(served_dish) = state.served_dish.take() {
                commands.entity(served_dish.entity).despawn();
            }

            events.push(SimEvent::DinerItemsChanged {
                entity: entity.to_entity_id(),
                change: DinerItemsChange::DropAll,
            });

            goal.update(DinerGoal::Leave);
        } else {
            goal.reset_timer();
        }
    }
}

/// Handles the diner leaving the canteen.
fn handle_leave_goal(
    mut commands: Commands,
    diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerTargets,
        &mut Movement,
    )>,
    canteen: Res<Canteen>,
) {
    for (entity, goal, mut targets, mut movement) in diner_query {
        if !goal.is(DinerGoal::Leave) {
            continue;
        }

        if targets.exit_target.is_none() {
            let best_exit = canteen
                .model
                .entrances
                .iter()
                .min_by_key(|xr| {
                    let distance = movement
                        .pos
                        .distance_squared(vec2(xr.center(), canteen.model.entrances_y));
                    NotNan::new(distance).unwrap()
                })
                .expect("should have at least one exit");

            let best_exit_area = Rect::new(
                best_exit.x_min + 0.3,
                canteen.model.entrances_y,
                best_exit.x_max - 0.3,
                canteen.model.entrances_y + 0.3,
            );
            targets.exit_target = Some(());
            movement.request_path_to_rect(best_exit_area);
            log::debug!(
                target: "diner",
                "leaving: pos={:.2} target={:?}",
                movement.pos, best_exit_area
            );
            continue;
        }

        if movement.target_reached {
            // Reached exit
            log::info!(
                target: "diner",
                "despawn: pos={:.2}",
                movement.pos
            );
            commands.entity(entity).despawn();
        }
    }
}

/// Finds a random valid (non-colliding) spot within a certain radius of a center point.
/// A simple utility for finding wandering targets.
fn find_valid_spot_near(
    center: Vec2,
    radius: Meters,
    nav_grid: &NavigationGrid,
    rng: &mut impl Rng,
) -> Vec2 {
    try_find_valid_spot_near(center, radius, nav_grid, rng).unwrap_or(center)
}

fn try_find_valid_spot_near(
    center: Vec2,
    radius: Meters,
    nav_grid: &NavigationGrid,
    rng: &mut impl Rng,
) -> Option<Vec2> {
    /// Attempts when searching for a valid (non-colliding) random spot
    pub const FIND_SPOT_ATTEMPTS: usize = 32;

    for _ in 0..FIND_SPOT_ATTEMPTS {
        let angle = rng.random_range(0.0..std::f32::consts::PI * 2.0);
        let distance = rng.random_range(radius * 0.5..radius);
        let point = center + Vec2::from_angle(angle) * distance;

        if nav_grid.is_pos_traversable(point, 0.5) {
            // here we use a loose body radius
            return Some(point);
        }
    }
    None
}

#[allow(unused)]
fn calculate_leave_probability(personality: &Personality) -> f32 {
    let patience_factor = 1.0 - (personality.patience_base / 120.0).min(1.0); // Normalize patience
    let adaptiveness_factor = 1.0 - personality.adaptiveness;
    let confrontational_factor = personality.confrontational;

    // Weighted average
    let base_prob =
        (patience_factor * 0.4 + adaptiveness_factor * 0.4 + confrontational_factor * 0.2)
            .clamp(0.0, 1.0);

    // Scale to reasonable range (0.05 - 0.4)
    0.05 + base_prob * 0.35
}
