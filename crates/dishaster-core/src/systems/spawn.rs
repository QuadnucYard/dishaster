use crate::{components::*, models::*, prelude::*, resources::*};

/// System that spawns all static objects (windows, tables, dispensers, collectors) at level start
pub fn spawn_static_objects(
    mut commands: Commands,
    _canteen: Res<Canteen>,
    level: Res<LevelConfig>,
    registry: Res<GameModelRegistry>,
) {
    // Spawn windows using new configuration
    for window_config in &level.window_configurations {
        let service_handle = registry
            .window_services
            .get_by_id(&window_config.service_template)
            .expect("Window service not found in registry");

        // Create active dishes from configuration
        let active_dishes: Vec<ActiveDish> = window_config
            .dish_assignments
            .iter()
            .map(|assignment| ActiveDish {
                assignment: assignment.clone(),
                state: DishRuntimeState {
                    current_quantity: assignment.initial_quantity,
                    current_quality: 0.8, // Default quality
                    contamination_level: 0.0,
                    last_restocked: 0.0,
                    service_count: 0,
                },
            })
            .collect();

        // Spawn window entity with separated data
        let window_entity = commands
            .spawn(Window {
                service_template: service_handle,
                config: window_config.clone(),
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
            .get_by_id(&table_placement.model)
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
            .get_by_id(&placement.model)
            .expect("Dispenser model not found in registry");
        let dispenser_model = registry.dispensers.get(dispenser_handle.clone());

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
            .get_by_id(&collector_placement.model)
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
    registry: Res<GameModelRegistry>,
    diner_query: Query<&Diner>,
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

    // Update spawn timer using the new time system
    // Update spawn timer using tick duration
    spawner.next_spawn_timer -= time.tick_duration;

    // Count current active diners (not used in simplified version)
    let _active_diners = diner_query.iter().count();

    // Check if we should spawn a new diner (simplified - no max limit)
    if spawner.next_spawn_timer <= 0.0 {
        // Generate new spawn interval using f64 for consistency
        let new_spawn_timer = rng.random_range(
            spawner.model.spawn_interval.min as f64..spawner.model.spawn_interval.max as f64,
        );

        spawn_diner_from_provider(
            &mut commands,
            &provider.model,
            &canteen.model,
            &registry,
            &mut rng,
        );

        // Reset spawn timer
        spawner.next_spawn_timer = new_spawn_timer;
    }
}

/// Generate a randomized diner model based on provider configuration ranges
pub fn generate_diner_model(provider_model: &DinerProviderModel, rng: &mut GameRng) -> DinerModel {
    DinerModel {
        attributes: DinerAttributes {
            hunger: rng.random_range(
                provider_model.attributes.hunger.min..provider_model.attributes.hunger.max,
            ),
            patience: rng.random_range(
                provider_model.attributes.patience.min..provider_model.attributes.patience.max,
            ),
            economic_capacity: rng.random_range(
                provider_model.attributes.economic_capacity.min
                    ..provider_model.attributes.economic_capacity.max,
            ),
            price_sensitivity: rng.random_range(
                provider_model.attributes.price_sensitivity.min
                    ..provider_model.attributes.price_sensitivity.max,
            ),
        },
        behavior: DinerBehavior {
            decisiveness: rng.random_range(
                provider_model.behavior.decisiveness.min..provider_model.behavior.decisiveness.max,
            ),
            adaptiveness: rng.random_range(
                provider_model.behavior.adaptiveness.min..provider_model.behavior.adaptiveness.max,
            ),
            leave_probability: rng.random_range(
                provider_model.behavior.leave_probability.min
                    ..provider_model.behavior.leave_probability.max,
            ),
            observation_time: rng.random_range(
                provider_model.behavior.observation_time.min
                    ..provider_model.behavior.observation_time.max,
            ),
            decision_time: rng.random_range(
                provider_model.behavior.decision_time.min
                    ..provider_model.behavior.decision_time.max,
            ),
            eating_time: rng.random_range(
                provider_model.behavior.eating_time.min..provider_model.behavior.eating_time.max,
            ),
        },
        properties: DinerProperties {
            base_satisfaction: 0.5, // Default base satisfaction
            preferences: vec![],    // Start with empty preferences
        },
    }
}

/// Spawn a new diner entity with randomized attributes at a canteen entrance
pub fn spawn_diner_from_provider(
    commands: &mut Commands,
    provider_model: &DinerProviderModel,
    canteen_model: &CanteenModel,
    _registry: &GameModelRegistry, // For future use when we have diner archetypes
    rng: &mut GameRng,
) {
    // For now, create a temporary diner model (later we'll use registry)
    let diner_model = generate_diner_model(provider_model, rng);

    // Generate random movement parameters
    let movement_speed = rng.random_range(
        provider_model.movement.movement_speed.min..provider_model.movement.movement_speed.max,
    );
    let avoidance_speed = rng.random_range(
        provider_model.movement.avoidance_speed.min..provider_model.movement.avoidance_speed.max,
    );
    let arrival_threshold = rng.random_range(
        provider_model.movement.arrival_threshold.min
            ..provider_model.movement.arrival_threshold.max,
    );

    // Get random entrance position
    let entrance = if canteen_model.entrances.is_empty() {
        XRange {
            x_min: -5.0,
            x_max: 5.0,
        }
    } else {
        let entrance_idx = rng.random_range(0..canteen_model.entrances.len());
        canteen_model.entrances[entrance_idx].clone()
    };

    let entrance_x = rng.random_range(entrance.x_min..entrance.x_max);
    let entrance_pos = Vec2::new(entrance_x, canteen_model.height + 2.0);

    let observation_x = rng.random_range(entrance.x_min..entrance.x_max);
    let observation_y = rng.random_range(2.0..canteen_model.height * 0.3);
    let observation_pos = Vec2::new(observation_x, observation_y);

    // Create diner entity with separated components - simplified for now
    commands.spawn((
        // For now, we'll use a placeholder until we set up proper archetype system
        DinerState {
            current: DinerStateType::Entering,
            state_timer: 0.0,
            satisfaction: diner_model.properties.base_satisfaction,
        },
        DinerTargets {
            chosen_window: None,
            chosen_table: None,
        },
        Movement {
            position: entrance_pos,
            target_position: observation_pos,
        },
        MovementModel {
            movement_speed,
            avoidance_speed,
            arrival_threshold,
        },
        DinerMemory {
            total_visits: 1,
            last_visit_day: 0,
            average_satisfaction: diner_model.properties.base_satisfaction,
            learned_preferences: vec![],
        },
        // Add the diner model directly as a component for now
        diner_model,
    ));
}

/// System to update the current diner count
pub fn check_day_completion(mut day_status: ResMut<DayStatus>, diner_query: Query<&Diner>) {
    // Update current diner count
    day_status.current_diner_count = diner_query.iter().count();
}
