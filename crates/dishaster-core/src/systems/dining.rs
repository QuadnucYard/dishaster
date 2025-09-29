use std::cmp::Ordering;

use dishaster_navigation::{NavigationGrid, PathRequest, find_path};

use crate::{components::*, constants::*, models::*, prelude::*, resources::*};

fn speed_factor_for_state(state: DinerStateType) -> f32 {
    match state {
        DinerStateType::MovingToSeat | DinerStateType::ReturningDishes => {
            CARRYING_TRAY_SPEED_FACTOR
        }
        _ => 1.0,
    }
}

/// Main diner behavior system - dispatches to state-specific handlers.
pub fn update_diner_states(
    mut commands: Commands,
    mut diner_query: Query<(
        Entity,
        &mut DinerState,
        &mut DinerTargets,
        &CompWrapper<DinerModel>,
        &mut Movement,
        Option<&QueueParticipant>,
    )>,
    window_query: Query<(Entity, &Window)>,
    mut table_set: ParamSet<(
        Query<(Entity, &DiningTable)>,
        Query<(Entity, &mut DiningTable)>,
    )>,
    collector_query: Query<(Entity, &DishCollector)>,
    canteen: Res<Canteen>,
    time: Res<Time>,
    mut rng: ResMut<GameRng>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
) {
    for (entity, mut state, mut targets, diner_model, mut movement, queue_participant) in
        diner_query.iter_mut()
    {
        // Update state timer using tick duration
        state.state_timer += time.tick_duration as f32;

        let next_state = match state.current {
            DinerStateType::Entering => handle_entering(&mut movement, &nav_grid, &mut rng),
            DinerStateType::Observing => handle_observing(
                &mut state,
                &mut targets,
                &mut movement,
                diner_model,
                &window_query,
                &canteen,
                &mut rng,
                &nav_grid,
            ),
            DinerStateType::Deciding => {
                handle_deciding(&mut state, &mut targets, diner_model, &mut rng)
            }
            DinerStateType::MovingToWindow => handle_moving_to_window(
                &mut movement,
                &mut targets,
                &window_query,
                &canteen,
                &nav_grid,
                queue_participant,
            ),
            DinerStateType::AtWindow => DinerStateType::Queueing,
            DinerStateType::Queueing => handle_queueing(&mut state, &movement, queue_participant),
            DinerStateType::BeingServed => handle_being_served(&mut state),
            DinerStateType::FindingSeat => handle_finding_seat(
                entity,
                &mut state,
                &mut targets,
                &mut movement,
                &nav_grid,
                &mut rng,
                &mut table_set,
            ),
            DinerStateType::MovingToSeat => handle_moving_to_seat(
                entity,
                &mut movement,
                &mut targets,
                &nav_grid,
                &mut table_set,
            ),
            DinerStateType::Eating => {
                handle_eating(entity, &mut state, &mut targets, &mut table_set)
            }
            DinerStateType::ReturningDishes => {
                handle_returning_dishes(&mut targets, &mut movement, &nav_grid, &collector_query)
            }
            DinerStateType::Leaving => {
                handle_leaving(&mut movement, &canteen, &nav_grid);
                DinerStateType::Leaving
            }
        };

        if next_state != state.current {
            if next_state == DinerStateType::MovingToWindow
                && state.current != DinerStateType::MovingToWindow
                && let Some(window) = targets.chosen_window
            {
                commands
                    .entity(entity)
                    .insert(QueueParticipant::new(window, time.current_time));
            }

            if !matches!(
                next_state,
                DinerStateType::MovingToWindow
                    | DinerStateType::Queueing
                    | DinerStateType::BeingServed
            ) && queue_participant.is_some()
            {
                commands.entity(entity).remove::<QueueParticipant>();
            }

            state.current = next_state;
            state.state_timer = 0.0;
        }

        movement.speed_factor = speed_factor_for_state(state.current);
    }
}

/// Handles the diner's entry into the canteen.
/// Sets their initial position and transitions them to observing.
fn handle_entering(
    movement: &mut Movement,
    nav_grid: &NavigationGrid,
    rng: &mut GameRng,
) -> DinerStateType {
    // Spawn already sets pos; here we ensure the first wander target is reasonable.
    let spot = find_valid_spot_near(movement.pos, WANDER_RADIUS, nav_grid, rng);
    let target_pos = spot;
    movement.compute_new_path(target_pos, nav_grid);

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
    nav_grid: &NavigationGrid,
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
            find_valid_spot_near(observation_center, WANDER_RADIUS, nav_grid, rng);
        // Clamp to bounds just in case
        let target_pos = clamp_to_canteen_with_margin(observation_spot, canteen);
        log::trace!(
            target: "nav",
            "observing: window={:?} target=({:.2},{:.2})",
            window_entity,
            target_pos.x,
            target_pos.y
        );

        movement.compute_new_path(target_pos, nav_grid);
        state.state_timer = 0.0; // Reset timer for new observation
    }

    // If the diner has reached their observation spot, transition to deciding.
    if movement
        .pos
        .close_to(movement.target_pos, OBSERVATION_ARRIVAL_EPS)
    {
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
    nav_grid: &NavigationGrid,
    queue_participant: Option<&QueueParticipant>,
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

    // Queue management system assigns the actual queue slot target.
    if queue_participant.is_none() {
        // Fallback: attempt to move near window center to avoid getting stuck.
        let fallback = Vec2::new(
            window.position.center(),
            (canteen.model.windows_y - WINDOW_APPROACH_OFFSET).clamp(0.0, canteen.model.height),
        );
        if movement.path.is_empty() || !movement.target_pos.close_to(fallback, 0.2) {
            movement.compute_new_path(fallback, nav_grid);
        }
        return DinerStateType::MovingToWindow;
    }

    if movement
        .pos
        .close_to(movement.target_pos, QUEUE_ARRIVAL_EPS)
    {
        log::trace!(
            target: "diner",
            "joined_queue: window={:?} pos=({:.2},{:.2})",
            window_entity,
            movement.pos.x,
            movement.pos.y
        );
        return DinerStateType::Queueing;
    }

    DinerStateType::MovingToWindow
}

/// Handles diners waiting in the queue until they reach the counter.
fn handle_queueing(
    _state: &mut DinerState,
    movement: &Movement,
    queue_participant: Option<&QueueParticipant>,
) -> DinerStateType {
    if let Some(queue) = queue_participant
        && queue.slot_index == 0
        && movement
            .pos
            .close_to(movement.target_pos, QUEUE_ARRIVAL_EPS)
    {
        log::trace!(target: "diner", "queue_front_reached");
        return DinerStateType::BeingServed;
    }

    // Remain in queue and keep accumulating wait time.
    DinerStateType::Queueing
}

/// Handles placeholder service timing once the diner reaches the counter.
fn handle_being_served(state: &mut DinerState) -> DinerStateType {
    if state.state_timer >= PLACEHOLDER_SERVICE_TIME_S {
        log::info!(target: "diner", "service_complete");
        DinerStateType::FindingSeat
    } else {
        DinerStateType::BeingServed
    }
}

fn handle_finding_seat(
    entity: Entity,
    state: &mut DinerState,
    targets: &mut DinerTargets,
    movement: &mut Movement,
    nav_grid: &NavigationGrid,
    rng: &mut GameRng,
    table_set: &mut ParamSet<(
        Query<(Entity, &DiningTable)>,
        Query<(Entity, &mut DiningTable)>,
    )>,
) -> DinerStateType {
    if let (Some(table_entity), Some(seat_index)) = (targets.chosen_table, targets.chosen_seat) {
        match table_set.p0().get(table_entity) {
            Ok((_, table))
                if table.occupants.get(seat_index).and_then(|slot| *slot) == Some(entity) =>
            {
                return DinerStateType::MovingToSeat;
            }
            _ => {
                targets.chosen_table = None;
                targets.chosen_seat = None;
            }
        }
    }

    let mut best: Option<(Entity, usize, bool, f32, f32, Vec2)> = None;
    for (table_entity, table) in table_set.p0().iter() {
        if table.seat_positions.is_empty() {
            continue;
        }
        let mut free_indices: Vec<usize> = table
            .occupants
            .iter()
            .enumerate()
            .filter_map(|(idx, occ)| if occ.is_none() { Some(idx) } else { None })
            .collect();
        if free_indices.is_empty() {
            continue;
        }
        free_indices.shuffle(rng);
        let seat_index = free_indices[0];
        let seat_pos = table.seat_positions[seat_index];
        let all_free = table.occupants.iter().all(|occ| occ.is_none());
        let dirtiness = table.dirtiness;
        let distance = movement.pos.distance_squared(seat_pos);

        let better = match &best {
            None => true,
            Some((_, _, best_all_free, best_dirtiness, best_distance, _)) => {
                match all_free.cmp(best_all_free) {
                    Ordering::Greater => true,
                    Ordering::Less => false,
                    Ordering::Equal => {
                        if dirtiness < *best_dirtiness - f32::EPSILON {
                            true
                        } else if dirtiness > *best_dirtiness + f32::EPSILON {
                            false
                        } else {
                            distance < *best_distance
                        }
                    }
                }
            }
        };

        if better {
            best = Some((
                table_entity,
                seat_index,
                all_free,
                dirtiness,
                distance,
                seat_pos,
            ));
        }
    }

    let Some((table_entity, seat_index, _, _, _, seat_pos)) = best else {
        if state.state_timer > MAX_SEAT_SEARCH_TIME_S {
            log::warn!(
                target: "diner",
                "finding_seat: timed out entity={:?}",
                entity
            );
            return DinerStateType::Leaving;
        }
        return DinerStateType::FindingSeat;
    };

    {
        let mut tables = table_set.p1();
        match tables.get_mut(table_entity) {
            Ok((_, mut table)) => {
                if matches!(table.occupants.get(seat_index), Some(slot) if slot.is_none()) {
                    table.occupants[seat_index] = Some(entity);
                } else {
                    return DinerStateType::FindingSeat;
                }
            }
            Err(_) => return DinerStateType::FindingSeat,
        }
    }

    targets.chosen_window = None;
    targets.observing_window = None;
    targets.chosen_table = Some(table_entity);
    targets.chosen_seat = Some(seat_index);
    targets.collector_target = None;

    movement.compute_new_path(seat_pos, nav_grid);

    log::trace!(
        target: "diner",
        "finding_seat: reserved table={:?} seat={} target=({:.2},{:.2})",
        table_entity,
        seat_index,
        movement.target_pos.x,
        movement.target_pos.y
    );

    DinerStateType::MovingToSeat
}

fn handle_moving_to_seat(
    entity: Entity,
    movement: &mut Movement,
    targets: &mut DinerTargets,
    nav_grid: &NavigationGrid,
    table_set: &mut ParamSet<(
        Query<(Entity, &DiningTable)>,
        Query<(Entity, &mut DiningTable)>,
    )>,
) -> DinerStateType {
    let (Some(table_entity), Some(seat_index)) = (targets.chosen_table, targets.chosen_seat) else {
        return DinerStateType::FindingSeat;
    };

    let seat_pos = {
        let tables = table_set.p0();
        match tables.get(table_entity) {
            Ok((_, table)) => {
                if table.occupants.get(seat_index).and_then(|slot| *slot) != Some(entity) {
                    targets.chosen_table = None;
                    targets.chosen_seat = None;
                    return DinerStateType::FindingSeat;
                }
                table.seat_positions[seat_index]
            }
            Err(_) => {
                targets.chosen_table = None;
                targets.chosen_seat = None;
                return DinerStateType::FindingSeat;
            }
        }
    };
    if movement.path.is_empty()
        && !movement
            .target_pos
            .close_to(seat_pos, TABLE_SEAT_ARRIVAL_EPS)
    {
        movement.compute_new_path(seat_pos, nav_grid);
    }

    if movement.pos.close_to(seat_pos, TABLE_SEAT_ARRIVAL_EPS) {
        movement.target_pos = seat_pos;
        movement.path.clear();
        movement.velocity = Vec2::ZERO;
        return DinerStateType::Eating;
    }

    DinerStateType::MovingToSeat
}

fn handle_eating(
    entity: Entity,
    state: &mut DinerState,
    targets: &mut DinerTargets,
    table_set: &mut ParamSet<(
        Query<(Entity, &DiningTable)>,
        Query<(Entity, &mut DiningTable)>,
    )>,
) -> DinerStateType {
    let (Some(table_entity), Some(seat_index)) = (targets.chosen_table, targets.chosen_seat) else {
        return DinerStateType::FindingSeat;
    };

    {
        let tables = table_set.p0();
        match tables.get(table_entity) {
            Ok((_, table))
                if table.occupants.get(seat_index).and_then(|slot| *slot) == Some(entity) => {}
            _ => {
                targets.chosen_table = None;
                targets.chosen_seat = None;
                return DinerStateType::FindingSeat;
            }
        }
    }

    if state.state_timer >= BASE_EATING_DURATION_S {
        let mut tables = table_set.p1();
        if let Ok((_, mut table)) = tables.get_mut(table_entity) {
            if matches!(
                table.occupants.get(seat_index),
                Some(slot) if slot.as_ref() == Some(&entity)
            ) {
                table.occupants[seat_index] = None;
            }
            table.dirtiness =
                (table.dirtiness + TABLE_DIRTINESS_INCREMENT).min(TABLE_MAX_DIRTINESS);
        }
        targets.chosen_table = None;
        targets.chosen_seat = None;
        return DinerStateType::ReturningDishes;
    }

    DinerStateType::Eating
}

fn handle_returning_dishes(
    targets: &mut DinerTargets,
    movement: &mut Movement,
    nav_grid: &NavigationGrid,
    collector_query: &Query<(Entity, &DishCollector)>,
) -> DinerStateType {
    if collector_query.is_empty() {
        return DinerStateType::Leaving;
    }

    if targets.collector_target.is_none() {
        let mut best: Option<(Entity, Vec2, f32)> = None;
        for (entity, collector) in collector_query.iter() {
            let distance = movement.pos.distance_squared(collector.center_pos);
            if best
                .as_ref()
                .map(|(_, _, best_distance)| distance < *best_distance)
                .unwrap_or(true)
            {
                best = Some((entity, collector.center_pos, distance));
            }
        }

        let Some((collector_entity, target_pos, _)) = best else {
            return DinerStateType::Leaving;
        };

        targets.collector_target = Some(collector_entity);
        movement.compute_new_path(target_pos, nav_grid);
        return DinerStateType::ReturningDishes;
    }

    let collector_entity = targets.collector_target.unwrap();
    let Ok((_, collector)) = collector_query.get(collector_entity) else {
        targets.collector_target = None;
        return DinerStateType::ReturningDishes;
    };

    let target_pos = collector.center_pos;
    if movement.pos.close_to(target_pos, COLLECTOR_ARRIVAL_EPS) {
        targets.collector_target = None;
        return DinerStateType::Leaving;
    }

    if movement.path.is_empty()
        || !movement
            .target_pos
            .close_to(target_pos, COLLECTOR_ARRIVAL_EPS)
    {
        movement.compute_new_path(target_pos, nav_grid);
    }

    DinerStateType::ReturningDishes
}

/// Handles the diner leaving the canteen.
fn handle_leaving(movement: &mut Movement, canteen: &Canteen, nav_grid: &NavigationGrid) {
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
        let exit_target = Vec2::new(
            exit_point.x.clamp(0.0, canteen.model.width),
            exit_point.y.clamp(0.0, canteen.model.height),
        );
        movement.compute_new_path(exit_target, nav_grid);
    }
}

impl Movement {
    /// Computes a new path to the specified target, updating the target position and path.
    /// If no valid path is found, the path is cleared.
    pub fn compute_new_path(&mut self, target: Vec2, nav_grid: &NavigationGrid) {
        if let Some(path) = find_path(PathRequest {
            start: self.pos,
            end: target,
            radius: self.radius,
            impatience: self.impatience,
            grid: nav_grid,
        }) {
            self.path = path;
        } else {
            self.path.clear();
        }
        self.target_pos = target;
    }
}

/// Finds a random valid (non-colliding) spot within a certain radius of a center point.
/// A simple utility for finding wandering targets.
fn find_valid_spot_near(
    center: Vec2,
    radius: Meters,
    nav_grid: &NavigationGrid,
    rng: &mut GameRng,
) -> Vec2 {
    /// Attempts when searching for a valid (non-colliding) random spot
    pub const FIND_SPOT_ATTEMPTS: usize = 12;

    for _ in 0..FIND_SPOT_ATTEMPTS {
        let angle = rng.random_range(0.0..std::f32::consts::PI * 2.0);
        let distance = rng.random_range(radius * 0.5..radius);
        let point = center + Vec2::from_angle(angle) * distance;

        if nav_grid.is_pos_traversable(point, 0.3) {
            // TODO: use actual diner radius
            return point;
        }
    }
    center // Fallback
}

/// Clamp a point to stay within the canteen bounds while keeping a soft margin from the walls.
///
/// This helper is intended for free-roaming targets (entering/observing). It must not be used
/// when queueing or interacting with service windows, where agents should hug the counters.
fn clamp_to_canteen_with_margin(point: Vec2, canteen: &Canteen) -> Vec2 {
    let margin = DINING_AREA_MARGIN;
    let width = canteen.model.width;
    let height = canteen.model.height;

    let (min_x, max_x) = if width <= margin * 2.0 {
        (0.0, width)
    } else {
        (margin, width - margin)
    };
    let (min_y, max_y) = if height <= margin * 2.0 {
        (0.0, height)
    } else {
        (margin, height - margin)
    };

    Vec2::new(point.x.clamp(min_x, max_x), point.y.clamp(min_y, max_y))
}

/*
struct PathPlan {
    path: Vec<Vec2>,
    goal: Vec2,
}

fn compute_path_with_fallback(
    start: Vec2,
    target: Vec2,
    canteen: &Canteen,
    nav_grid: &NavigationGrid,
) -> Option<PathPlan> {
    if let Some(path) = find_path(PathRequest {
        start,
        end: target,
        grid: nav_grid,
        world_width: canteen.model.width,
        world_height: canteen.model.height,
        crowd: None,
    }) {
        return Some(PathPlan { path, goal: target });
    }

    if let Some(open_point) = nearest_open_point(target, canteen, nav_grid, 6)
        && let Some(path) = find_path(PathRequest {
            start,
            end: open_point,
            grid: nav_grid,
            world_width: canteen.model.width,
            world_height: canteen.model.height,
            crowd: None,
        })
    {
        return Some(PathPlan {
            path,
            goal: open_point,
        });
    }

    None
}

fn nearest_open_point(
    target: Vec2,
    canteen: &Canteen,
    nav_grid: &NavigationGrid,
    max_radius: i32,
) -> Option<Vec2> {
    let target_tile = nav_grid.world_to_grid(target);
    if !nav_grid.is_occupied(target_tile) {
        return Some(target);
    }

    for radius in 1..=max_radius {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs_diff(radius) != 0 && dy.abs_diff(radius) != 0 {
                    continue;
                }
                let tile = target_tile + IVec2::new(dx, dy);
                let world = nav_grid.tile_to_world(tile);
                if world.x < 0.0
                    || world.x > canteen.model.width
                    || world.y < 0.0
                    || world.y > canteen.model.height
                {
                    continue;
                }
                if !nav_grid.is_occupied(tile) {
                    return Some(world);
                }
            }
        }
    }

    None
}

fn direct_path_fallback(start: Vec2, target: Vec2) -> Vec<Vec2> {
    if start.distance_squared(target) < f32::EPSILON {
        Vec::new()
    } else {
        vec![target]
    }
}
 */
