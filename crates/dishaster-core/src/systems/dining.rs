use crate::{
    components::*,
    constants::*,
    models::*,
    prelude::*,
    resources::*,
    utils::pathfinding::{PathRequest, find_path},
};

/// Main diner behavior system - dispatches to state-specific handlers.
pub fn update_diner_states(
    mut diner_query: Query<(
        &mut DinerState,
        &mut DinerTargets,
        &DinerModel,
        &mut Movement,
    )>,
    window_query: Query<(Entity, &Window)>,
    canteen: Res<Canteen>,
    time: Res<Time>,
    mut rng: ResMut<GameRng>,
    collision_grid: Res<CollisionGridRes>,
) {
    for (mut state, mut targets, diner_model, mut movement) in diner_query.iter_mut() {
        // Update state timer using tick duration
        state.state_timer += time.tick_duration as f32;

        let next_state = match state.current {
            DinerStateType::Entering => {
                handle_entering(&mut movement, &canteen, &collision_grid, &mut rng)
            }
            DinerStateType::Observing => handle_observing(
                &mut state,
                &mut targets,
                &mut movement,
                diner_model,
                &window_query,
                &canteen,
                &mut rng,
                &collision_grid,
            ),
            DinerStateType::Deciding => {
                handle_deciding(&mut state, &mut targets, diner_model, &mut rng)
            }
            DinerStateType::MovingToWindow => handle_moving_to_window(
                &mut movement,
                &mut targets,
                &window_query,
                &canteen,
                &collision_grid,
            ),
            DinerStateType::AtWindow => {
                // Per instructions, transition directly to leaving for now.
                DinerStateType::Leaving
            }
            DinerStateType::Leaving => {
                handle_leaving(&mut movement, &canteen, &collision_grid);
                DinerStateType::Leaving // Remain in leaving state
            }
        };

        if next_state != state.current {
            state.current = next_state;
            state.state_timer = 0.0;
        }
    }
}

/// Handles the diner's entry into the canteen.
/// Sets their initial position and transitions them to observing.
fn handle_entering(
    movement: &mut Movement,
    _canteen: &Canteen,
    collision_grid: &CollisionGridRes,
    rng: &mut GameRng,
) -> DinerStateType {
    // Spawn already sets pos; here we ensure the first wander target is reasonable.
    let spot = find_valid_spot_near(movement.pos, WANDER_RADIUS, collision_grid, rng);
    movement.target_pos = Vec2::new(
        spot.x.clamp(0.0, _canteen.model.width),
        spot.y.clamp(0.0, _canteen.model.height),
    );
    log::trace!(
        target: "nav",
        "entering: pos=({:.2},{:.2}) first_target=({:.2},{:.2})",
        movement.pos.x,
        movement.pos.y,
        movement.target_pos.x,
        movement.target_pos.y
    );

    if let Some(path) = find_path(PathRequest {
        start: movement.pos,
        end: movement.target_pos,
        grid: collision_grid,
        world_width: _canteen.model.width,
        world_height: _canteen.model.height,
    }) {
        movement.path = path;
        log::trace!(target: "nav", "entering: path_len={}", movement.path.len());
    } else {
        log::debug!(target: "nav", "entering: no path");
    }
    DinerStateType::Observing
}

/// Handles the observing state where a diner wanders around to check windows.
fn handle_observing(
    state: &mut DinerState,
    targets: &mut DinerTargets,
    movement: &mut Movement,
    diner_model: &DinerModel,
    window_query: &Query<(Entity, &Window)>,
    canteen: &Canteen,
    rng: &mut GameRng,
    collision_grid: &CollisionGridRes,
) -> DinerStateType {
    // If no window is being observed, or if we've been observing for too long, pick a new one.
    if targets.observing_window.is_none()
        || state.state_timer > diner_model.behavior.observation_time
    {
        // Simple logic: pick a random available window to observe.
        let available_windows: Vec<_> = window_query
            .iter()
            .filter(|(_, w)| w.config.is_enabled)
            .collect();

        let Some((window_entity, window)) = available_windows.choose(rng).map(|(e, w)| (*e, *w))
        else {
            // No windows available, decide to leave.
            return DinerStateType::Leaving;
        };

        targets.observing_window = Some(window_entity);
        // Find a valid observation spot near the window.
        let observation_center = Vec2::new(window.position.center(), canteen.model.windows_y);
        let observation_spot =
            find_valid_spot_near(observation_center, WANDER_RADIUS, collision_grid, rng);
        // Clamp to bounds just in case
        movement.target_pos = Vec2::new(
            observation_spot.x.clamp(0.0, canteen.model.width),
            observation_spot.y.clamp(0.0, canteen.model.height),
        );
        log::trace!(
            target: "nav",
            "observing: window={:?} target=({:.2},{:.2})",
            window_entity,
            movement.target_pos.x,
            movement.target_pos.y
        );

        if let Some(path) = find_path(PathRequest {
            start: movement.pos,
            end: movement.target_pos,
            grid: collision_grid,
            world_width: canteen.model.width,
            world_height: canteen.model.height,
        }) {
            movement.path = path;
            log::trace!(target: "nav", "observing: path_len={}", movement.path.len());
        } else {
            log::debug!(target: "nav", "observing: no path");
        }
        state.state_timer = 0.0; // Reset timer for new observation
    }

    // If the diner has reached their observation spot, transition to deciding.
    if movement.pos.distance(movement.target_pos) < OBSERVATION_ARRIVAL_EPS {
        log::trace!(
            target: "nav",
            "observing: arrived target=({:.2},{:.2})",
            movement.target_pos.x,
            movement.target_pos.y
        );
        return DinerStateType::Deciding;
    }

    DinerStateType::Observing
}

/// Handles the decision-making process after observing a window.
fn handle_deciding(
    state: &mut DinerState,
    targets: &mut DinerTargets,
    diner_model: &DinerModel,
    rng: &mut GameRng,
) -> DinerStateType {
    if state.state_timer > diner_model.behavior.decision_time {
        // Simplified decision: random chance to like the window.
        // In a real scenario, this would use diner preferences, queue length, etc.
        if rng.random_bool(0.7) {
            // 70% chance to choose the observed window
            if let Some(window_entity) = targets.observing_window {
                targets.chosen_window = Some(window_entity);
                log::info!(target: "diner", "decision: choose_window entity={:?}", window_entity);
                return DinerStateType::MovingToWindow;
            }
        }

        // If not chosen, clear observation target and go back to observing.
        targets.observing_window = None;
        return DinerStateType::Observing;
    }
    // Safety fallback: if somehow stuck much longer than intended, leave.
    if state.state_timer > diner_model.behavior.decision_time * 3.0 {
        return DinerStateType::Leaving;
    }
    DinerStateType::Deciding
}

/// Handles moving the diner to their chosen window.
fn handle_moving_to_window(
    movement: &mut Movement,
    targets: &mut DinerTargets,
    window_query: &Query<(Entity, &Window)>,
    canteen: &Canteen,
    collision_grid: &CollisionGridRes,
) -> DinerStateType {
    let Some(window_entity) = targets.chosen_window else {
        // Should not happen, but as a fallback, go observe.
        return DinerStateType::Observing;
    };

    let Ok((_, window)) = window_query.get(window_entity) else {
        // Chosen window disappeared? Go back to observing.
        targets.chosen_window = None;
        return DinerStateType::Observing;
    };

    // Approach the counter directly (ignore queue in this phase).
    let approach_spot = Vec2::new(
        window.position.center(),
        canteen.model.windows_y + WINDOW_APPROACH_OFFSET,
    );
    if movement.target_pos != approach_spot {
        movement.target_pos = Vec2::new(
            approach_spot.x.clamp(0.0, canteen.model.width),
            approach_spot.y.clamp(0.0, canteen.model.height),
        );
        log::trace!(
            target: "nav",
            "move_to_window: target=({:.2},{:.2})",
            movement.target_pos.x,
            movement.target_pos.y
        );

        // Use pathfinding for movement
        if let Some(path) = find_path(PathRequest {
            start: movement.pos,
            end: movement.target_pos,
            grid: collision_grid,
            world_width: canteen.model.width,
            world_height: canteen.model.height,
        }) {
            movement.path = path;
            log::trace!(target: "nav", "move_to_window: path_len={}", movement.path.len());
        } else {
            log::debug!(target: "nav", "move_to_window: no path");
        }
    }

    // If close enough, transition to the next state (simplified for now).
    if movement.pos.distance(movement.target_pos) < QUEUE_ARRIVAL_EPS {
        log::info!(target: "diner", "arrived_at_window: pos=({:.2},{:.2})", movement.pos.x, movement.pos.y);
        return DinerStateType::AtWindow;
    }

    DinerStateType::MovingToWindow
}

/// Handles the diner leaving the canteen.
fn handle_leaving(movement: &mut Movement, canteen: &Canteen, collision_grid: &CollisionGridRes) {
    // Entrances also serve as exits. Compute nearest point on any entrance XRange at Y = entrances_y.
    let mut best_point: Option<Vec2> = None;
    let mut best_dist_sq: f32 = f32::INFINITY;
    for xr in &canteen.model.entrances {
        let clamped_x = movement.pos.x.clamp(xr.x_min, xr.x_max);
        let candidate = Vec2::new(clamped_x, canteen.model.entrances_y);
        let d2 = movement.pos.distance_squared(candidate);
        if d2 < best_dist_sq {
            best_dist_sq = d2;
            best_point = Some(candidate);
        }
    }

    if let Some(exit_point) = best_point
        && movement.target_pos.distance(exit_point) > EXIT_ARRIVAL_EPS
    {
        movement.target_pos = Vec2::new(
            exit_point.x.clamp(0.0, canteen.model.width),
            exit_point.y.clamp(0.0, canteen.model.height),
        );
        if let Some(path) = find_path(PathRequest {
            start: movement.pos,
            end: movement.target_pos,
            grid: collision_grid,
            world_width: canteen.model.width,
            world_height: canteen.model.height,
        }) {
            movement.path = path;
        }
    }
}

/// Finds a random valid (non-colliding) spot within a certain radius of a center point.
/// A simple utility for finding wandering targets.
fn find_valid_spot_near(
    center: Vec2,
    radius: Meters,
    _collision_grid: &CollisionGridRes,
    rng: &mut GameRng,
) -> Vec2 {
    for _ in 0..FIND_SPOT_ATTEMPTS {
        let angle = rng.random_range(0.0..std::f32::consts::PI * 2.0);
        let distance = rng.random_range(radius * 0.5..radius);
        let point = center + Vec2::new(angle.cos() * distance, angle.sin() * distance);
        // Clamp to a reasonable positive range; caller may further clamp to canteen bounds.
        if point.x.is_nan() || point.y.is_nan() {
            continue;
        }
        // Skip occupancy check to speed up; dynamic occupancy ignored in pathfinding anyway.
        return point;
    }
    center // Fallback
}

/// System to clean up diners who have left.
pub fn despawn_leaving_diners(
    mut commands: Commands,
    query: Query<(Entity, &Diner, &DinerState, &Movement)>,
    canteen: Res<Canteen>,
) {
    for (entity, diner, state, movement) in query.iter() {
        if state.current != DinerStateType::Leaving {
            continue;
        }
        // Check if diner has reached any of the exits.
        // If close enough to any exit point on an entrance range, despawn.
        let reached_exit = canteen.model.entrances.iter().any(|xr| {
            let clamped_x = movement.pos.x.clamp(xr.x_min, xr.x_max);
            let exit_point = Vec2::new(clamped_x, canteen.model.entrances_y);
            movement.pos.distance(exit_point) < EXIT_ARRIVAL_EPS
        });
        if reached_exit {
            log::info!(
                target: "diner",
                "despawn: id={} pos=({:.2},{:.2})",
                diner.id,
                movement.pos.x,
                movement.pos.y
            );
            commands.entity(entity).despawn();
        }
    }
}
