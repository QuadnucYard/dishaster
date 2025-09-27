//! Static canteen layout spawning systems

use dishrupt_core::{
    display::{DisplayState, Transform},
    utils::Modified,
};

use crate::{components::*, constants::*, models::*, prelude::*, resources::*};

/// System that spawns all static objects (windows, tables, dispensers, collectors) at level start
pub fn spawn_static_objects(
    mut commands: Commands,
    canteen: Res<Canteen>,
    level: Res<LevelConfigRes>,
    registry: Res<GameModelRegistryRes>,
    display_root: Res<DisplayRoot>,
) {
    spawn_windows(&mut commands, &canteen, &level, &registry);
    spawn_tables(&mut commands, &level, &registry, &display_root);
    spawn_dispensers(&mut commands, &level, &registry);
    spawn_collectors(&mut commands, &level, &registry);
}

fn spawn_windows(
    commands: &mut Commands,
    canteen: &Res<Canteen>,
    level: &Res<LevelConfigRes>,
    registry: &Res<GameModelRegistryRes>,
) {
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
}

fn spawn_tables(
    commands: &mut Commands,
    level: &Res<LevelConfigRes>,
    registry: &Res<GameModelRegistryRes>,
    display_root: &DisplayRoot,
) {
    for placement in &level.table_placements {
        let handle = registry
            .tables
            .get_handle_by_id(&placement.model)
            .expect("Table model not found in registry");
        let model = registry.tables.get(handle);
        let seat_positions = (0..model.seats)
            .map(|i| {
                let local_x = (i as f32 + 0.5) / model.seats as f32 * model.size.width; // relative to top-left
                placement.center_pos + Vec2::new(local_x, 0.0) - model.size.as_vec2() / 2.0
            })
            .collect();
        commands.spawn((
            DiningTable {
                model: handle,
                center_pos: placement.center_pos,
                seat_positions,
                occupants: vec![None; model.seats],
                dirtiness: 0.0,
            },
            BoxCollider(dishaster_navigation::BoxCollider {
                center: placement.center_pos,
                size: model.size.as_vec2(),
            }),
            DisplayState {
                proto: model.display.res.clone(),
                ..Default::default()
            },
            Transform {
                position: placement.center_pos.extend(0.0),
                parent: Modified::new(Some(display_root.0)),
                ..Default::default()
            },
        ));
    }
}

fn spawn_dispensers(
    commands: &mut Commands,
    level: &Res<LevelConfigRes>,
    registry: &Res<GameModelRegistryRes>,
) {
    // Spawn tray dispensers
    for dispenser_placement in &level.tray_dispenser_placements {
        spawn_dispenser(commands, registry, dispenser_placement, DispenserType::Tray);
    }

    // Spawn chopstick dispensers
    for dispenser_placement in &level.chopstick_dispenser_placements {
        spawn_dispenser(
            commands,
            registry,
            dispenser_placement,
            DispenserType::Chopstick,
        );
    }
}

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

fn spawn_collectors(
    commands: &mut Commands,
    level: &Res<LevelConfigRes>,
    registry: &GameModelRegistry,
) {
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
