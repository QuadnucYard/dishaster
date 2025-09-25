use dishrupt_core::{
    display::{DisplayModel, DisplayState, Transform},
    utils::Modified,
};

use crate::{components::*, constants::*, models::*, prelude::*, resources::*};

/// System to update the current diner count
pub fn check_day_completion(mut day_status: ResMut<DayStatus>, diner_query: Query<&Diner>) {
    // Update current diner count
    day_status.current_diner_count = diner_query.iter().count();
}

/// System that spawns all static objects (windows, tables, dispensers, collectors) at level start
pub fn spawn_static_objects(
    mut commands: Commands,
    canteen: Res<Canteen>,
    level: Res<LevelConfigRes>,
    registry: Res<GameModelRegistryRes>,
) {
    // Spawn windows using new configuration
    for window_config in &level.window_configurations {
        let service_handle = registry
            .window_services
            .get_handle_by_id(&window_config.service_template)
            .expect("Window service not found in registry");

        // Create active dishes from configuration
        let active_dishes: Vec<ActiveDish> = window_config
            .dish_assignments
            .iter()
            .map(|assignment| ActiveDish {
                assignment: assignment.clone(),
                state: DishRuntimeState {
                    current_quantity: DEFAULT_DISH_QUANTITY,
                    current_quality: DEFAULT_DISH_QUALITY,
                    contamination_level: DEFAULT_DISH_CONTAMINATION,
                    last_restocked: DEFAULT_DISH_LAST_RESTOCKED_S,
                    service_count: 0,
                },
            })
            .collect();

        // Spawn window entity with separated data
        let window_entity = commands
            .spawn(Window {
                service_template: service_handle,
                config: window_config.clone(),
                position: canteen.model.windows[window_config.slot_index],
            })
            .id();

        // Add dishes as separate component for better data locality
        commands.entity(window_entity).insert(WindowDishes {
            dishes: active_dishes,
        });
    }

    // Spawn tables
    for table_placement in &level.table_placements {
        let table_handle = registry
            .tables
            .get_handle_by_id(&table_placement.model)
            .expect("Table model not found in registry");
        commands.spawn(DiningTable {
            model: table_handle,
            center_pos: table_placement.center_pos,
            occupied: [false; 2],
            dirtiness: 0.0,
        });
    }

    // Helper function to spawn dispensers
    fn spawn_dispenser(
        commands: &mut Commands,
        registry: &GameModelRegistry,
        placement: &DispenserPlacement,
        dispenser_type: DispenserType,
    ) {
        let dispenser_handle = registry
            .dispensers
            .get_handle_by_id(&placement.model)
            .expect("Dispenser model not found in registry");
        let dispenser_model = registry.dispensers.get(dispenser_handle);

        commands.spawn(Dispenser {
            model: dispenser_handle,
            center_pos: placement.center_pos,
            current_stock: dispenser_model.initial_stock,
            dispenser_type,
        });
    }

    // Spawn tray dispensers
    for dispenser_placement in &level.tray_dispenser_placements {
        spawn_dispenser(
            &mut commands,
            &registry,
            dispenser_placement,
            DispenserType::Tray,
        );
    }

    // Spawn chopstick dispensers
    for dispenser_placement in &level.chopstick_dispenser_placements {
        spawn_dispenser(
            &mut commands,
            &registry,
            dispenser_placement,
            DispenserType::Chopstick,
        );
    }

    // Spawn dish collectors
    for collector_placement in &level.collector_placements {
        let collector_handle = registry
            .collectors
            .get_handle_by_id(&collector_placement.model)
            .expect("Dish collector model not found in registry");
        commands.spawn(DishCollector {
            model: collector_handle,
            center_pos: collector_placement.center_pos,
            current_load: 0,
        });
    }
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

    // Check if we should spawn a new diner
    if spawner.next_spawn_timer <= 0.0 {
        // Generate new spawn interval
        let new_spawn_timer = rng
            .random_range(spawner.model.spawn_interval.min..spawner.model.spawn_interval.max)
            as f64;

        let diner_model = generate_diner_model(&provider.model, &mut rng);

        spawn_diner(
            diner_model,
            &mut commands,
            &canteen,
            &mut spawner,
            &display_root,
            &mut rng,
        );

        // Reset spawn timer
        spawner.next_spawn_timer = new_spawn_timer;
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
                velocity: Vec2::ZERO,
                path: Vec::new(),
                last_pos: pos,
                ignoring_collisions: false,
            },
        },
        BoxCollider(dishaster_navigation::BoxCollider {
            center: pos,
            size: Vec2::new(DINER_COLLIDER_SIZE, DINER_COLLIDER_SIZE),
        }),
        DisplayState {
            proto: model.display.res.clone(),
            ..Default::default()
        },
        Transform {
            position: Vec3::new(pos.x, pos.y, 0.0),
            parent: Modified::new(Some(display_root.0)),
            ..Default::default()
        },
        DinerModelComp::from(model),
    ));
}
