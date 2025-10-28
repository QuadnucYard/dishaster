//! Static canteen layout spawning systems

use dishaster_navigation::BoxCollider;
use dishrupt_core::{
    asset::PrefabReference,
    display::{DisplayState, Transform},
};

use crate::systems::prelude::*;

const PRICE_LABEL_OFFSET: Vec3 = vec3(0.0, 0.2, 0.05);
const PRICE_LABEL_PREFAB: &str = "dishes/price_label";

/// System that spawns all static objects (windows, tables, dispensers, collectors) at level start
pub fn spawn_static_objects(
    mut commands: Commands,
    canteen: Res<Canteen>,
    level: Res<ResWrapper<LevelConfig>>,
    registry: Res<GameModelRegistryRes>,
    display_root: Res<DisplayRoot>,
    mut events: ResMut<EventLog>,
) {
    spawn_windows(
        &mut commands,
        &canteen,
        &level,
        &registry,
        &display_root,
        &mut events,
    );
    spawn_tables(&mut commands, &level, &registry, &display_root);
    spawn_dispensers(&mut commands, &level, &registry, &display_root);
    spawn_collectors(&mut commands, &level, &registry, &display_root);
}

fn spawn_windows(
    commands: &mut Commands,
    canteen: &Res<Canteen>,
    level: &LevelConfig,
    registry: &GameModelRegistry,
    display_root: &DisplayRoot,
    events: &mut ResMut<EventLog>,
) {
    let mut last_window_x = 0.0;

    for (i, window_config) in level.window_configurations.iter().enumerate() {
        let service_handle = registry
            .window_services
            .get_handle_by_id(&window_config.service_template)
            .expect("Window service not found in registry");
        let service_template = registry.window_services.get(service_handle);

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
        let window_range = canteen.model.windows[window_config.slot_index];
        let window_location = XSegment::new(
            window_range.center() - service_template.layout.size.width / 2.0,
            window_range.center() + service_template.layout.size.width / 2.0,
            canteen.model.windows_y,
        );
        let window_entity = commands
            .spawn((
                Window {
                    service_template: service_handle,
                    slot_index: window_config.slot_index,
                    location: window_location,
                    enabled: window_config.is_enabled,
                },
                DisplayState {
                    proto: service_template.display.res.clone(),
                    name: Some(eco_format!("Window_{}", i)),
                    ..Default::default()
                },
                Transform {
                    position: window_location.center().extend(0.0), // align by top-center
                    parent: Some(display_root.0),
                    ..Default::default()
                },
            ))
            .id();

        spawn_dish_presentations(
            commands,
            window_entity,
            &active_dishes,
            service_template,
            registry,
            events,
        );

        // Add dishes as separate component for better data locality
        commands.entity(window_entity).insert(WindowDishes {
            dishes: active_dishes,
        });

        // Fill spaces between windows with colliders
        commands.spawn(
            BoxCollider::from_rect(Rect::new(
                last_window_x,
                canteen.model.windows_y,
                window_range.x_min,
                canteen.model.height,
            ))
            .into_comp(),
        );
        last_window_x = window_range.x_max;
    }

    commands.spawn(
        BoxCollider::from_rect(Rect::new(
            last_window_x,
            canteen.model.windows_y,
            canteen.model.width,
            canteen.model.height,
        ))
        .into_comp(),
    );

    // Add a small gap to isolate the hall
    commands.spawn(
        BoxCollider::from_rect(Rect::new(
            0.0,
            canteen.model.windows_y - 0.05,
            canteen.model.width,
            canteen.model.windows_y + 0.05,
        ))
        .into_comp(),
    );
}

fn spawn_dish_presentations(
    commands: &mut Commands,
    window_entity: Entity,
    dishes: &[ActiveDish],
    service_template: &WindowServiceModel,
    registry: &GameModelRegistry,
    events: &mut ResMut<EventLog>,
) {
    if dishes.is_empty() {
        return;
    }

    let layout = &service_template.layout;
    if layout.dish_slots.is_empty() {
        return;
    }

    for active in dishes {
        let slot_index = active.assignment.slot_index;
        let Some(slot_rect) = layout.dish_slots.get(slot_index) else {
            continue;
        };

        let dish_handle = registry
            .dishes
            .get_handle_by_id(&active.assignment.dish_id)
            .expect("Dish not found in registry");
        let dish_model = registry.dishes.get(dish_handle);

        let wrapper_entity = commands
            .spawn((
                DisplayState {
                    name: Some(eco_format!("WindowDish_Slot{}", slot_index)),
                    ..Default::default()
                },
                Transform {
                    position: (slot_rect.center() - vec2(layout.size.width / 2.0, 0.0)).extend(0.0),
                    parent: Some(window_entity),
                    ..Default::default()
                },
                ChildOf(window_entity),
            ))
            .id();

        commands.spawn((
            DisplayState {
                proto: dish_model.display.res.clone(),
                ..Default::default()
            },
            Transform {
                parent: Some(wrapper_entity),
                ..Default::default()
            },
            ChildOf(wrapper_entity),
        ));

        commands.spawn((
            DisplayState {
                proto: PrefabReference::new(PRICE_LABEL_PREFAB),
                name: Some("Price".into()), // required for referencing in scripts
                ..Default::default()
            },
            Transform {
                position: PRICE_LABEL_OFFSET,
                parent: Some(wrapper_entity),
                ..Default::default()
            },
            ChildOf(wrapper_entity),
        ));

        events.emit(PresentationEvent::DishSpawned(DishViewModel {
            entity: wrapper_entity.into(),
            dish_id: active.assignment.dish_id.clone(),
            pricing: active.assignment.pricing.method,
        }));
    }
}

fn spawn_tables(
    commands: &mut Commands,
    level: &LevelConfig,
    registry: &GameModelRegistry,
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
                placement.center_pos - model.size.as_vec2() / 2.0 + Vec2::new(local_x, -0.5)
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
            BoxCollider::from_center_size(placement.center_pos, model.size.as_vec2()).into_comp(),
            DisplayState {
                proto: model.display.res.clone(),
                ..Default::default()
            },
            Transform {
                position: placement.center_pos.extend(0.0),
                parent: Some(display_root.0),
                ..Default::default()
            },
        ));
    }
}

fn spawn_dispensers(
    commands: &mut Commands,
    level: &LevelConfig,
    registry: &GameModelRegistry,
    display_root: &DisplayRoot,
) {
    for (placements, ty) in [
        (&level.tray_dispenser_placements, DispenserType::Tray),
        (
            &level.chopstick_dispenser_placements,
            DispenserType::Chopstick,
        ),
    ] {
        for dispenser_placement in placements {
            spawn_dispenser(commands, registry, dispenser_placement, ty, display_root);
        }
    }
}

fn spawn_dispenser(
    commands: &mut Commands,
    registry: &GameModelRegistry,
    placement: &DispenserPlacement,
    dispenser_type: DispenserType,
    display_root: &DisplayRoot,
) {
    let dispenser_handle = registry
        .dispensers
        .get_handle_by_id(&placement.model)
        .expect("Dispenser model not found in registry");
    let model = registry.dispensers.get(dispenser_handle);

    commands.spawn((
        Dispenser {
            model: dispenser_handle,
            center_pos: placement.center_pos,
            reception_area: Rect::from_center_size(
                placement.center_pos + model.reception_area.center(),
                model.reception_area.size(),
            ),
            current_stock: model.initial_stock,
            dispenser_type,
        },
        BoxCollider::from_center_size(placement.center_pos, model.size.as_vec2()).into_comp(),
        DisplayState {
            proto: model.display.res.clone(),
            ..Default::default()
        },
        Transform {
            position: placement.center_pos.extend(0.0),
            parent: Some(display_root.0),
            ..Default::default()
        },
    ));
}

fn spawn_collectors(
    commands: &mut Commands,
    level: &LevelConfig,
    registry: &GameModelRegistry,
    display_root: &DisplayRoot,
) {
    // Spawn dish collectors
    for placement in &level.collector_placements {
        let collector_handle = registry
            .collectors
            .get_handle_by_id(&placement.model)
            .expect("Dish collector model not found in registry");
        let model = registry.collectors.get(collector_handle);
        commands.spawn((
            DishCollector {
                model: collector_handle,
                center_pos: placement.center_pos,
                reception_area: Rect::from_center_size(
                    placement.center_pos + model.reception_area.center(),
                    model.reception_area.size(),
                ),
                current_load: 0,
            },
            BoxCollider::from_center_size(placement.center_pos, model.size.as_vec2()).into_comp(),
            DisplayState {
                proto: model.display.res.clone(),
                ..Default::default()
            },
            Transform {
                position: placement.center_pos.extend(0.0),
                parent: Some(display_root.0),
                ..Default::default()
            },
        ));
    }
}
