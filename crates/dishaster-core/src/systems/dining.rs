use crate::{components::*, models::*, prelude::*, resources::*};

/// Main diner behavior system - handles state machine updates
pub fn update_diner_states(
    mut diner_query: Query<(Entity, &mut DinerState, &mut DinerTargets, &DinerModel)>,
    mut movement_query: Query<&mut Movement>,
    mut table_query: Query<(Entity, &mut DiningTable)>,
    window_query: Query<(Entity, &Window)>,
    db: Res<GameModelRegistryRes>,
    canteen: Res<Canteen>,
    time: Res<Time>,
    mut rng: ResMut<GameRng>,
) {
    for (entity, mut state, mut targets, diner_model) in diner_query.iter_mut() {
        // Get movement component separately to avoid borrow conflicts
        let mut movement = movement_query.get_mut(entity).unwrap();

        // Update state timer using tick duration
        state.state_timer += time.tick_duration as f32;

        match state.current {
            DinerStateType::Entering => {
                // Move to observation point near windows
                movement.target_position = Vec2::new(120.0, 80.0);
                // Direct movement without collision avoidance
                movement.position = movement.target_position;
                state.current = DinerStateType::Observing;
                state.state_timer = 0.0;
            }
            DinerStateType::Observing => {
                let observation_time = diner_model.behavior.observation_time;
                if state.state_timer > observation_time {
                    state.current = DinerStateType::Deciding;
                    state.state_timer = 0.0;
                }
            }
            DinerStateType::Deciding => {
                if state.state_timer > diner_model.behavior.decision_time {
                    // Random decision based on probability
                    if rng.random_bool(diner_model.behavior.leave_probability as f64) {
                        state.current = DinerStateType::Leaving;
                    } else {
                        // Find available window
                        for (window_entity, window) in window_query.iter() {
                            if window.config.is_enabled {
                                let service = db.window_services.get(window.service_template);
                                targets.chosen_window = Some(window_entity);
                                let queue_x = window.position.x_min
                                    + *service.layout.queue_x.choose(&mut rng).unwrap();
                                movement.target_position =
                                    Vec2::new(queue_x, canteen.model.windows_y);
                                state.current = DinerStateType::MovingToWindow;
                                break;
                            }
                        }

                        // If no window available, leave
                        if targets.chosen_window.is_none() {
                            state.current = DinerStateType::Leaving;
                        }
                    }
                    state.state_timer = 0.0;
                }
            }
            DinerStateType::MovingToWindow => {
                // Direct movement to chosen window
                movement.position = movement.target_position;
                state.current = DinerStateType::BeingServed;
                state.state_timer = 0.0;
            }
            DinerStateType::BeingServed => {
                if state.state_timer > 5.0 {
                    // Fixed service time for now
                    state.current = DinerStateType::LookingForTable;
                    state.state_timer = 0.0;
                }
            }
            DinerStateType::LookingForTable => {
                let pause_time = rng.random_range(1.0..3.0);

                if state.state_timer > pause_time {
                    // Look for available table
                    for (table_entity, table) in table_query.iter() {
                        if table.occupied.iter().any(|&occupied| !occupied) {
                            targets.chosen_table = Some(table_entity);
                            movement.target_position = Vec2::new(200.0, 200.0);
                            state.current = DinerStateType::MovingToTable;
                            break;
                        }
                    }

                    // If no table found, wait a bit and try again
                    if targets.chosen_table.is_none() {
                        state.state_timer = 0.0;
                    }
                }
                // Small random movement during pause
                else if state.state_timer > pause_time * 0.5 {
                    let random_angle = rng.random_range(0.0..(2.0 * std::f32::consts::PI));
                    let look_distance = 5.0;
                    movement.target_position = movement.position
                        + Vec2::new(
                            random_angle.cos() * look_distance,
                            random_angle.sin() * look_distance,
                        );
                    movement.position = movement.target_position;
                }
            }
            DinerStateType::MovingToTable => {
                if let Some(table_entity) = targets.chosen_table {
                    movement.position = movement.target_position;
                    state.current = DinerStateType::EatingAtTable;
                    state.state_timer = 0.0;
                    // Mark table as occupied
                    if let Ok((_, mut table)) = table_query.get_mut(table_entity) {
                        for i in 0..table.occupied.len() {
                            if !table.occupied[i] {
                                table.occupied[i] = true;
                                break;
                            }
                        }
                    }
                } else {
                    state.current = DinerStateType::Leaving;
                }
            }
            DinerStateType::EatingAtTable => {
                let eating_time = diner_model.behavior.eating_time;
                if state.state_timer > eating_time {
                    state.current = DinerStateType::ReturningPlate;
                    // Free up table
                    if let Some(table_entity) = targets.chosen_table
                        && let Ok((_, mut table)) = table_query.get_mut(table_entity)
                    {
                        for i in 0..table.occupied.len() {
                            if table.occupied[i] {
                                table.occupied[i] = false;
                                break;
                            }
                        }
                    }
                    state.state_timer = 0.0;
                }
            }
            DinerStateType::ReturningPlate => {
                // Move to plate return area
                movement.target_position = Vec2::new(250.0, 100.0);
                movement.position = movement.target_position;
                state.current = DinerStateType::Leaving;
            }
            DinerStateType::Leaving => {
                // Direct movement to exit
                movement.target_position = Vec2::new(400.0, 0.0);
                movement.position = movement.target_position;
                // Mark for despawn when reached exit
                if movement.position.distance(movement.target_position) < 1.0 {
                    // Will be handled by despawn system
                }
            }
        }
    }
}

/// System to clean up diners who have left
pub fn despawn_leaving_diners(
    mut commands: Commands,
    query: Query<(Entity, &DinerState, &Movement)>,
) {
    for (entity, state, movement) in query.iter() {
        if let DinerStateType::Leaving = state.current {
            // Check if diner has reached the exit
            let exit_pos = Vec2::new(400.0, 0.0);
            if movement.position.distance(exit_pos) < 2.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}
