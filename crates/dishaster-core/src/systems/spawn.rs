mod layout;
mod queues;
mod staffs;

use bevy_ecs::schedule::ScheduleConfigs;
use dishrupt_core::{
    asset::PrefabReference,
    display::{DisplayModel, DisplayState, Transform},
};
pub use layout::*;
pub use queues::*;
pub use staffs::*;

use super::prelude::*;

/// Initial spawning systems to run at level start
pub fn initial_spawning_systems() -> ScheduleConfigs<Box<dyn System<In = (), Out = ()> + 'static>> {
    (
        spawn_static_objects,
        spawn_window_queues,
        spawn_serving_staffs,
    )
        .chain()
}

/// System that manages diner spawning based on timing and capacity constraints
pub fn update_diner_spawner(
    mut commands: Commands,
    time: Res<Time>,
    day_status: Res<DayStatus>,
    mut spawner: ResMut<DinerSpawner>,
    provider: Res<DinerProvider>,
    canteen: Res<Canteen>,
    display_root: Res<DisplayRoot>,
    mut rng: ResMut<GameRng>,
) {
    if !day_status.started {
        // Wait for the service phase to begin before spawning diners.
        return;
    }

    // Don't spawn new diners if spawning is finished
    if spawner.spawning_finished {
        return;
    }
    if spawner.is_spawning_complete(time.current_time) {
        spawner.spawning_finished = true;
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
        Vec2::new(x, canteen.model.entrances_y + 0.5)
    };

    spawner.next_diner_id += 1;

    log::info!(
        target: "diner",
        "spawn: id={} pos={:.2}",
        spawner.next_diner_id,
        pos
    );

    let diner_id = spawner.next_diner_id;
    let display_res = model.display.res.clone();
    let wrapper = commands.spawn((
        AgentTag,
        DinerBundle {
            diner: Diner { id: diner_id },
            goal: DinerGoalState::default(),
            targets: DinerTargets::default(),
            movement: Movement {
                pos,
                radius: rng.random_range(0.2..0.4),
                impatience: rng.random_range(0.5..1.0), // TODO: base on model
                avoidance_responsibility: rng.random_range(1.0..3.0),
                ..Default::default()
            },
        },
        model.into_comp(),
        DisplayState {
            name: Some(eco_format!("Diner_{}", diner_id)),
            ..Default::default()
        },
        Transform {
            position: pos.extend(0.0),
            parent: Some(display_root.0),
            ..Default::default()
        },
    ));
    let wrapper_entity = wrapper.id();

    let _body = commands.spawn((
        DisplayState {
            proto: display_res,
            ..Default::default()
        },
        Transform {
            position: Vec3::ZERO,
            parent: Some(wrapper_entity),
            ..Default::default()
        },
        ChildOf(wrapper_entity),
    ));

    let _feedback = commands.spawn((
        DisplayState {
            proto: PrefabReference::new("feedback_balloon"),
            name: Some("Feedback".into()),
            ..Default::default()
        },
        Transform {
            position: vec3(0.0, 0.0, 1.7),
            parent: Some(wrapper_entity),
            ..Default::default()
        },
        ChildOf(wrapper_entity),
    ));
}
