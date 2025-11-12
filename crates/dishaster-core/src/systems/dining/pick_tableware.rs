use crate::systems::{feedback::*, prelude::*};

pub fn handle_pick_tray_goal(
    mut commands: Commands,
    diner_query: Query<(
        Entity,
        &mut DinerState,
        &mut DinerGoalState,
        &mut DinerTargets,
        &mut Movement,
        &mut DinerPsychState,
        &DinerPersonality,
        &mut EntityRng,
    )>,
    mut dispenser_query: Query<(Entity, &Dispenser, &mut Stock)>,
    registry: Res<GameModelRegistryRes>,
    time: Res<Time>,
    mut feedback_messages: MessageWriter<FeedbackMessage>,
    mut events: ResMut<EventQueue>,
) {
    const DISPENSER_RETRY_COOLDOWN: f32 = 5.0; // Retry every 5 seconds instead of every frame

    for (
        entity,
        mut state,
        mut goal,
        mut targets,
        mut movement,
        mut psych_state,
        _personality,
        mut rng,
    ) in diner_query
    {
        if !goal.is(DinerGoal::PickTray) {
            continue;
        }

        if targets.tray_target.is_none() {
            // Rate limit: only search for new dispenser after cooldown
            let current_time = time.current_time as f32;
            if current_time - targets.last_dispenser_retry_time < DISPENSER_RETRY_COOLDOWN {
                continue;
            }

            // Choose the closest tray dispenser with stock
            let Some((dispenser_entity, dispenser, stock)) = dispenser_query
                .iter()
                .filter(|(_, d, s)| {
                    d.dispenser_type == DispenserType::Tray
                        && (!d.center_pos.close_to(movement.pos, 3.0) || s.current > 0)
                })
                .min_by_key(|(_, d, _)| {
                    let distance = movement.pos.distance_squared(d.center_pos);
                    NotNan::new(distance).unwrap()
                })
            else {
                // No dispensers available - emit feedback and wait for cooldown
                log::info!(
                    target: "diner",
                    "diner {:?} cannot find tray with stock, waiting for cooldown",
                    entity
                );

                // Update last retry time
                targets.last_dispenser_retry_time = current_time;

                // Apply mood penalty (but not trust - this could be temporary)
                psych_state.mood = (psych_state.mood - 0.05).max(-1.0);

                // Emit complaint feedback
                if goal.timer > 10.0 {
                    // Only emit feedback if we've been trying for a while
                    feedback_messages.write(FeedbackMessage {
                        entity,
                        content: choose_feedback(&mut rng, feedbacks::MISSING_UTENSILS),
                        trigger: Some(FeedbackTrigger::MissingUtensils),
                    });
                    goal.reset_timer(); // Reset to avoid spamming feedback
                }

                // Don't leave yet - keep trying
                continue;
            };
            targets.tray_target = Some(dispenser_entity);
            targets.last_dispenser_retry_time = current_time;
            log::debug!(
                target: "diner",
                "picking_tray: target={dispenser_entity:?}, stock={}",
                stock.current
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
        let Ok((_, dispenser, mut stock)) = dispenser_query.get_mut(tray_dispenser_entity) else {
            continue;
        };

        // Check if dispenser still has stock
        if stock.current == 0 {
            log::warn!(
                target: "diner",
                "tray_dispenser_empty: entity={entity:?}, dispenser={tray_dispenser_entity:?}"
            );
            // Dispenser is empty, find another one with rate limiting
            targets.tray_target = None;
            targets.last_dispenser_retry_time = time.current_time as f32;
            continue;
        }

        // Deduct stock
        stock.current -= 1;

        let dispenser_model = registry.dispensers.get(dispenser.model);

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

pub fn handle_pick_chopsticks_goal(
    mut commands: Commands,
    diner_query: Query<(
        Entity,
        &mut DinerState,
        &mut DinerGoalState,
        &mut DinerTargets,
        &mut Movement,
        &mut DinerPsychState,
        &mut EntityRng,
    )>,
    mut dispenser_query: Query<(Entity, &Dispenser, &mut Stock)>,
    registry: Res<GameModelRegistryRes>,
    time: Res<Time>,
    mut feedback_messages: MessageWriter<FeedbackMessage>,
    mut events: ResMut<EventQueue>,
) {
    const DISPENSER_RETRY_COOLDOWN: f32 = 5.0; // Retry every 5 seconds instead of every frame

    for (entity, mut state, mut goal, mut targets, mut movement, mut psych_state, mut rng) in
        diner_query
    {
        if !goal.is(DinerGoal::PickChopsticks) {
            continue;
        }

        if targets.chopstick_target.is_none() {
            // Rate limit: only search for new dispenser after cooldown
            let current_time = time.current_time as f32;
            if current_time - targets.last_dispenser_retry_time < DISPENSER_RETRY_COOLDOWN {
                continue;
            }

            // Choose the closest chopstick dispenser with stock
            let Some((dispenser_entity, dispenser, stock)) = dispenser_query
                .iter()
                .filter(|(_, d, s)| {
                    d.dispenser_type == DispenserType::Chopstick
                        && (!d.center_pos.close_to(movement.pos, 3.0) || s.current > 0)
                })
                .min_by_key(|(_, d, _)| {
                    let distance = movement.pos.distance_squared(d.center_pos);
                    NotNan::new(distance).unwrap()
                })
            else {
                // No dispensers available - wait for cooldown
                log::info!(
                    target: "diner",
                    "diner {:?} cannot find chopsticks with stock, waiting for cooldown",
                    entity
                );

                // Update last retry time
                targets.last_dispenser_retry_time = current_time;

                // Apply mood penalty (but not trust - this could be temporary)
                psych_state.mood = (psych_state.mood - 0.05).max(-1.0);

                // Emit complaint feedback
                if goal.timer > 10.0 {
                    // Only emit feedback if we've been trying for a while
                    feedback_messages.write(FeedbackMessage {
                        entity,
                        content: choose_feedback(&mut rng, feedbacks::MISSING_UTENSILS),
                        trigger: Some(FeedbackTrigger::MissingUtensils),
                    });
                    goal.reset_timer(); // Reset to avoid spamming feedback
                }

                // Don't leave yet - keep trying
                continue;
            };
            targets.chopstick_target = Some(dispenser_entity);
            targets.last_dispenser_retry_time = current_time;
            log::debug!(
                target: "diner",
                "pick_chopsticks_target: target={dispenser_entity:?}, stock={}",
                stock.current
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

        let Ok((_, dispenser, mut stock)) = dispenser_query.get_mut(chopstick_dispenser_entity)
        else {
            continue;
        };

        // Check if dispenser still has stock
        if stock.current == 0 {
            log::warn!(
                target: "diner",
                "chopstick_dispenser_empty: entity={entity:?}, dispenser={chopstick_dispenser_entity:?}"
            );
            // Dispenser is empty, find another one with rate limiting
            targets.chopstick_target = None;
            targets.last_dispenser_retry_time = time.current_time as f32;
            continue;
        }

        // Deduct stock
        stock.current -= 1;

        let dispenser_model = registry.dispensers.get(dispenser.model);

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

        goal.update(if !state.served_dishes.is_empty() {
            DinerGoal::FindSeat
        } else {
            log::debug!("Chopsticks picked before being served!");
            DinerGoal::QueueForWindow
        });
    }
}

pub fn handle_return_dishes_goal(
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

            // Despawn all served dishes
            for served_dish in state.served_dishes.drain(..) {
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
