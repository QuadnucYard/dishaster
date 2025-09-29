mod layout;

use dishrupt_core::{
    display::{DisplayModel, DisplayState, Transform},
    utils::Modified,
};
pub use layout::spawn_static_objects;

use crate::{components::*, constants::*, models::*, prelude::*, resources::*};

/// System to update the current diner count
pub fn check_day_completion(mut day_status: ResMut<DayStatus>, diner_query: Query<&Diner>) {
    // Update current diner count
    day_status.current_diner_count = diner_query.iter().count();
}

/// System that manages diner spawning based on timing and capacity constraints
pub fn update_diner_spawner(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<DinerSpawner>,
    provider: Res<DinerProvider>,
    canteen: Res<Canteen>,
    display_root: Res<DisplayRoot>,
    mut rng: ResMut<GameRng>,
) {
    // Check if spawning time is finished using the new time system
    if spawner.is_spawning_complete(time.current_time) {
        spawner.spawning_finished = true;
        return;
    }

    // Don't spawn new diners if spawning is finished
    if spawner.spawning_finished {
        return;
    }

    // Update spawn timer using tick duration
    spawner.next_spawn_timer -= time.tick_duration;

    while spawner.next_spawn_timer <= 0.0 {
        let diner_model = generate_diner_model(&provider.model, &mut rng);

        spawn_diner(
            diner_model,
            &mut commands,
            &canteen,
            &mut spawner,
            &display_root,
            &mut rng,
        );

        // Schedule next spawn using exponential sampling around current time
        let interval = spawner.sample_next_interval(&mut rng, time.current_time);
        spawner.next_spawn_timer += interval;

        if spawner.is_spawning_complete(time.current_time) {
            break;
        }
    }
}

/// Generate a randomized diner model based on provider configuration ranges
fn generate_diner_model(provider: &DinerProviderModel, rng: &mut GameRng) -> DinerModel {
    DinerModel {
        attributes: DinerAttributes {
            hunger: rng
                .random_range(provider.attributes.hunger.min..provider.attributes.hunger.max),
            patience: rng
                .random_range(provider.attributes.patience.min..provider.attributes.patience.max),
            economic_capacity: rng.random_range(
                provider.attributes.economic_capacity.min
                    ..provider.attributes.economic_capacity.max,
            ),
            price_sensitivity: rng.random_range(
                provider.attributes.price_sensitivity.min
                    ..provider.attributes.price_sensitivity.max,
            ),
        },
        behavior: DinerBehavior {
            decisiveness: rng.random_range(
                provider.behavior.decisiveness.min..provider.behavior.decisiveness.max,
            ),
            adaptiveness: rng.random_range(
                provider.behavior.adaptiveness.min..provider.behavior.adaptiveness.max,
            ),
            leave_probability: rng.random_range(
                provider.behavior.leave_probability.min..provider.behavior.leave_probability.max,
            ),
            observation_time: rng.random_range(
                provider.behavior.observation_time.min..provider.behavior.observation_time.max,
            ),
            decision_time: rng.random_range(
                provider.behavior.decision_time.min..provider.behavior.decision_time.max,
            ),
            eating_time: rng
                .random_range(provider.behavior.eating_time.min..provider.behavior.eating_time.max),
        },
        properties: DinerProperties {
            base_satisfaction: 0.5, // Default base satisfaction
            preferences: vec![],    // Start with empty preferences
        },
        display: DisplayModel {
            res: provider
                .display_res
                .choose(rng)
                .cloned()
                .unwrap_or_default(),
            ..Default::default()
        },
    }
}

/// Spawn a new diner entity with randomized attributes at a canteen entrance
fn spawn_diner(
    model: DinerModel,
    commands: &mut Commands,
    canteen: &Canteen,
    spawner: &mut ResMut<DinerSpawner>,
    display_root: &DisplayRoot,
    rng: &mut GameRng,
) {
    // Sample a spawn X uniformly from the configured entrance X ranges.
    // Y coordinate is fixed at entrances_y. All in meters.
    let pos = {
        let x_range = canteen.model.entrances.choose(rng).unwrap();
        let x = rng.random_range(x_range.x_min..x_range.x_max);
        Vec2::new(x, canteen.model.entrances_y)
    };

    spawner.next_diner_id += 1;

    log::info!(
        target: "diner",
        "spawn: id={} pos=({:.2},{:.2})",
        spawner.next_diner_id,
        pos.x,
        pos.y
    );

    commands.spawn((
        DinerBundle {
            diner: Diner {
                id: spawner.next_diner_id,
            },
            state: DinerState {
                current: DinerStateType::Entering,
                state_timer: 0.0,
                satisfaction: DEFAULT_DINER_SATISFACTION,
            },
            targets: DinerTargets::default(),
            movement: Movement {
                pos,
                target_pos: pos,
                next_waypoint: pos,
                radius: rng.random_range(0.2..0.4),
                impatience: rng.random_range(0.0..1.0),
                ..Default::default()
            },
        },
        DisplayState {
            proto: model.display.res.clone(),
            ..Default::default()
        },
        Transform {
            position: pos.extend(0.0),
            parent: Modified::new(Some(display_root.0)),
            ..Default::default()
        },
        DinerModelComp::from(model),
    ));
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
            movement.pos.close_to(exit_point, EXIT_ARRIVAL_EPS)
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
