//! Diner behavior and state machine system.

mod decide_window;
mod deciding;
mod eating;
mod ordering;
mod pick_tableware;
mod queue;

use bevy_ecs::schedule::ScheduleConfigs;
use dishaster_navigation::NavigationGrid;
use ordered_float::NotNan;

use self::{decide_window::*, deciding::*, eating::*, pick_tableware::*, queue::*};
use crate::systems::{feedback::*, prelude::*};

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
    mut feedback_messages: MessageWriter<FeedbackMessage>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
    time: Res<Time>,
) {
    let dt = time.tick_duration;

    for (entity, mut goal, mut targets, mut movement, personality, mut rng) in diner_query {
        if !goal.is(DinerGoal::Observe) {
            continue;
        }

        // If no window is being observed, or if we've been observing for too long, pick a new one.
        if targets.observing_window.is_none()
            || !movement.has_path() && goal.timer > 5.0 / personality.decisiveness.max(0.1)
        {
            // Simple logic: pick a random available window to observe.
            let available_windows = window_query.iter().filter(|(_, w)| !w.disabled);

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

        // Occasionally show observing feedback (rate: 0.01/s means ~1% per second)
        if rng.random_bool_dt(0.01, dt) {
            feedback_messages.write(FeedbackMessage {
                entity,
                content: choose_feedback(&mut rng, feedbacks::OBSERVING),
                trigger: None,
            });
        }
    }
}

fn handle_get_served_goal(
    mut commands: Commands,
    diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerState,
        &DinerTargets,
        &ServiceSession,
    )>,
    time: Res<Time>,
    mut daily_stats: ResMut<DailyStats>,
) {
    for (entity, mut goal, mut state, targets, session) in diner_query {
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

        // Mark serving end time and record to daily stats
        state.serving_end_time = Some(time.current_time as f32);
        if let (Some(start), Some(end)) = (state.serving_start_time, state.serving_end_time) {
            let serving_duration = end - start;
            daily_stats.serving_times.push(serving_duration);
            log::debug!(
                target: "diner",
                "diner {} serving time: {:.1}s",
                entity,
                serving_duration
            );
        }

        // Service completed, remove session and move to finding a seat
        commands
            .entity(entity)
            .remove::<QueueMember>()
            .remove::<ServiceSession>();

        if state.served_dishes.is_empty() {
            log::error!(
                target: "diner",
                "diner {} has no served dishes after service completion!",
                entity
            );
            continue;
        }

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
    time: Res<Time>,
) {
    let dt = time.tick_duration;

    for (mut goal, mut targets, mut movement, mut rng) in diner_query {
        if !goal.is(DinerGoal::FindSeat) {
            continue;
        }

        // After 3s cooldown, attempt to find seat with rate 0.1/s (~9.5% chance per second)
        if !(goal.timer > 3.0 && rng.random_bool_dt(0.1, dt)) {
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
        &mut DinerState,
        &mut DinerTargets,
        &mut Movement,
    )>,
    mut table_query: Query<(Entity, &mut DiningTable)>,
    time: Res<Time>,
    mut events: ResMut<EventQueue>,
) {
    for (entity, mut goal, mut state, mut targets, mut movement) in diner_query {
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
                change: DinerItemsChange::StartEating(table_entity.to_entity_id(), seat_index),
            });

            // Mark dining start time
            state.dining_start_time = Some(time.current_time as f32);

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

/// Handles the diner leaving the canteen.
fn handle_leave_goal(
    mut commands: Commands,
    diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerState,
        &mut DinerTargets,
        &mut Movement,
    )>,
    canteen: Res<Canteen>,
) {
    for (entity, goal, mut state, mut targets, mut movement) in diner_query {
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

            despawn_diner_items(&mut commands, &mut state);
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
