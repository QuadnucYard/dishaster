//! Diner behavior and state machine system.

use bevy_ecs::schedule::ScheduleConfigs;
use dishaster_navigation::NavigationGrid;
use ordered_float::NotNan;

use super::{feedback::*, prelude::*};

/// Collection of schedules dining systems
pub fn dining_systems() -> ScheduleConfigs<Box<dyn System<In = (), Out = ()> + 'static>> {
    (
        update_diner_goals,
        (
            handle_enter_goal,
            handle_observe_goal,
            handle_decide_goal,
            handle_queue_for_window_goal,
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
    for (mut goal,) in diner_query {
        goal.step(time.tick_duration as f32);
    }
}

fn handle_enter_goal(
    diner_query: Query<(&mut DinerGoalState, &mut Movement)>,
    mut rng: ResMut<GameRng>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
) {
    for (mut goal, mut movement) in diner_query {
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
        &CompWrapper<DinerModel>,
    )>,
    window_query: Query<(Entity, &Window)>,
    time: Res<Time>,
    mut rng: ResMut<GameRng>,
    mut events: ResMut<EventLog>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
) {
    for (entity, mut goal, mut targets, mut movement, diner_model) in diner_query {
        if !goal.is(DinerGoal::Observe) {
            continue;
        }

        // If no window is being observed, or if we've been observing for too long, pick a new one.
        if targets.observing_window.is_none()
            || goal.timer > diner_model.behavior.observation_time && !movement.has_path()
        {
            // Simple logic: pick a random available window to observe.
            let available_windows = window_query.iter().filter(|(_, w)| w.config.is_enabled);

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
            events.emit_feedback(FeedbackEvent {
                entity: entity.into(),
                content: Feedback::Thought(choose_feedback(&mut rng, OBSERVING_FEEDBACKS).into()),
                timestamp: time.current_time,
            })
        }
    }
}

fn handle_decide_goal(
    mut commands: Commands,
    diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerTargets,
        &CompWrapper<DinerModel>,
    )>,
    lane_query: Query<(Entity, &QueueLaneMembers)>,
    window_query: Query<&LaneOwner, With<Window>>,
    time: Res<Time>,
    mut rng: ResMut<GameRng>,
    mut events: ResMut<EventLog>,
) {
    for (entity, mut goal, mut targets, diner_model) in diner_query {
        if !goal.is(DinerGoal::DecideWindow) {
            continue;
        }

        if goal.timer < diner_model.behavior.decision_time {
            continue;
        }

        // Simplified decision: random chance to like the window.
        // In a real scenario, this would use diner preferences, queue length, etc.
        // 70% chance to choose the observed window
        let Some(window_entity) = targets.observing_window else {
            // If not chosen, clear observation target and go back to observing.
            goal.update(DinerGoal::Observe);
            continue;
        };

        if rng.random_bool(0.7) {
            log::info!(target: "diner", "decision: choose_window entity={window_entity:?}");

            // Give feedback
            if rng.random_bool(0.5) {
                events.emit_feedback(FeedbackEvent {
                    entity: entity.into(),
                    content: Feedback::Thought(
                        choose_feedback(&mut rng, DECIDING_FEEDBACKS).into(),
                    ),
                    timestamp: time.current_time,
                });
            }

            targets.chosen_window = Some(window_entity);

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

            goal.update(DinerGoal::QueueForWindow);

            continue;
        }
    }
}

fn handle_queue_for_window_goal(diner_query: Query<(&mut DinerGoalState, &QueueMember)>) {
    for (mut goal, queue_member) in diner_query {
        if !goal.is(DinerGoal::QueueForWindow) {
            continue;
        }

        if queue_member.ranking == 0 {
            goal.update(DinerGoal::GetServed);
            // todo: start session
            continue;
        }
    }
}

fn handle_find_seat_goal(
    diner_query: Query<(&mut DinerGoalState, &mut DinerTargets, &mut Movement)>,
    table_query: Query<(Entity, &mut DiningTable)>,
    mut rng: ResMut<GameRng>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
) {
    for (mut goal, mut targets, mut movement) in diner_query {
        if !goal.is(DinerGoal::FindSeat) {
            continue;
        }

        if goal.timer > 3.0 && rng.random_bool(0.1) {
            if let Some((table_entity, seat_index)) =
                find_seat(movement.pos, &table_query, &mut rng)
            {
                let table = table_query.get(table_entity).unwrap().1;
                targets.chosen_seat = Some((table_entity, seat_index));
                movement.request_path(table.seat_positions[seat_index]);
                log::debug!(
                    target: "diner",
                    "found_seat: table={table_entity:?} seat={seat_index} target={:.2}",
                    table.seat_positions[seat_index]
                );
                goal.update(DinerGoal::MoveToSeat);
                continue;
            } else {
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
                continue;
            }
        }
    }
}

fn find_seat(
    pos: Vec2,
    table_query: &Query<(Entity, &mut DiningTable)>,
    rng: &mut GameRng,
) -> Option<(Entity, usize)> {
    // scoring factors (lower is better):
    // - distance
    // - dirtiness
    // - occupancy

    let (table_entity, table) = table_query
        .iter()
        .filter(|(_, table)| table.occupants.iter().any(|o| o.is_none()))
        .max_by_key(|(_, table)| {
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
) {
    for (entity, mut goal, mut targets, mut movement) in diner_query {
        if !goal.is(DinerGoal::MoveToSeat) {
            continue;
        }

        let Some((table_entity, seat_index)) = targets.chosen_seat else {
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
            // Seat taken, need to find another
            targets.chosen_seat = None;
            goal.update(DinerGoal::FindSeat);
            continue;
        }

        // Check reaching the seat
        let seat_pos = table.seat_positions[seat_index];
        if movement.pos.close_to(seat_pos, TABLE_SEAT_ARRIVAL_EPS) {
            movement.stop();
            movement.pos = seat_pos; // snap to seat position
            table.occupants[seat_index] = Some(entity); // Mark the seat as occupied
            goal.update(DinerGoal::Eat);
            continue;
        }
    }
}

fn handle_eat_goal(
    diner_query: Query<(
        &mut DinerGoalState,
        &mut DinerTargets,
        &CompWrapper<DinerModel>,
    )>,
    mut table_query: Query<(Entity, &mut DiningTable)>,
    mut rng: ResMut<GameRng>,
) {
    for (mut goal, mut targets, diner_model) in diner_query {
        if !goal.is(DinerGoal::Eat) {
            continue;
        }

        if goal.timer < diner_model.behavior.eating_time {
            continue;
        }

        // Finished eating
        let (table_entity, seat_index) = targets.chosen_seat.expect("should have chosen seat");
        let mut table = table_query
            .get_mut(table_entity)
            .expect("table should exist")
            .1;
        table.dirtiness += rng.random_range(0.01..0.2); // increase dirtiness
        table.occupants[seat_index] = None; // Free the seat
        targets.chosen_seat = None;

        goal.update(DinerGoal::ReturnDishes);
    }
}

fn handle_return_dishes_goal(
    diner_query: Query<(&mut DinerGoalState, &mut DinerTargets, &mut Movement)>,
    collector_query: Query<(Entity, &DishCollector)>,
) {
    if collector_query.is_empty() {
        return;
    }

    for (mut goal, mut targets, mut movement) in diner_query {
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
            movement.request_path(collector.center_pos); // fixme: should plan a path close to that point
            log::debug!(
                target: "diner",
                "returning_dishes: target={collector_entity:?} pos={:.2}",
                collector.center_pos
            );
            continue;
        }

        if movement.target_reached {
            targets.collector_target = None;

            goal.update(DinerGoal::Leave);
        }
    }
}

/// Handles the diner leaving the canteen.
fn handle_leave_goal(
    mut commands: Commands,
    diner_query: Query<(Entity, &mut DinerGoalState, &mut Movement)>,
    canteen: Res<Canteen>,
    mut rng: ResMut<GameRng>,
) {
    for (entity, goal, mut movement) in diner_query {
        if !goal.is(DinerGoal::Leave) {
            continue;
        }

        if !movement.has_path() {
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
            let best_exit_point = vec2(
                if best_exit.width() > 0.6 {
                    rng.random_range((best_exit.x_min + 0.3)..(best_exit.x_max - 0.3))
                } else {
                    best_exit.center()
                },
                canteen.model.entrances_y,
            );

            movement.request_path(best_exit_point);
            log::debug!(
                target: "diner",
                "leaving: pos={:.2} target={:.2}",
                movement.pos,
                best_exit_point
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
    rng: &mut GameRng,
) -> Vec2 {
    try_find_valid_spot_near(center, radius, nav_grid, rng).unwrap_or(center)
}

fn try_find_valid_spot_near(
    center: Vec2,
    radius: Meters,
    nav_grid: &NavigationGrid,
    rng: &mut GameRng,
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
