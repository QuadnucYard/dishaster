use crate::{
    events::DispatchManagement,
    systems::{prelude::*, spawn_table},
};

pub fn register_management_decision_systems(world: &mut World) {
    macro_rules! add_observers {
        ($($system: ident),* $(,)?) => {
            $( world.add_observer($system); )*
        };
    }

    add_observers!(
        apply_add_tables,
        apply_remove_tables,
        apply_disarrange_tables,
        apply_open_window,
        apply_close_window,
        apply_change_window_service,
    );
}

/// Add tables at random positions
fn apply_add_tables(
    event: On<DispatchManagement<AddTablesModel>>,
    mut commands: Commands,
    registry: Res<GameModelRegistryRes>,
    canteen: Res<Canteen>,
    display_root: Res<DisplayRoot>,
    mut rng: ResMut<WorldRng>,
) {
    let model = &event.0;
    for _ in 0..model.num_tables {
        // Spawn at random position
        let table_model_id = registry
            .tables
            .keys()
            .choose(&mut rng)
            .expect("No table models in registry")
            .clone();
        let center_pos = random_table_position(&canteen.model, &mut rng);

        // TODO: check for collisions with existing tables
        let placement = Placement {
            model: table_model_id,
            center_pos,
        };
        spawn_table(&placement, &mut commands, &registry, &display_root);
    }
}

/// Remove random tables
fn apply_remove_tables(
    event: On<DispatchManagement<RemoveTablesModel>>,
    mut commands: Commands,
    table_query: Query<Entity, With<DiningTable>>,
    mut rng: ResMut<WorldRng>,
) {
    let model = &event.0;
    for entity in table_query
        .iter()
        .choose_multiple(&mut rng, model.num_tables)
    {
        commands.entity(entity).despawn();
    }
}

/// Remove reposition tables
fn apply_disarrange_tables(
    event: On<DispatchManagement<DisarrangeTablesModel>>,
    mut table_query: Query<(Entity, &mut DiningTable, &mut Transform)>,
    canteen: Res<Canteen>,
    mut rng: ResMut<WorldRng>,
) {
    let model = &event.0;
    for (_, mut table, mut transform) in table_query
        .iter_mut()
        .choose_multiple(&mut rng, model.num_tables)
    {
        let new_pos = random_table_position(&canteen.model, &mut rng);
        table.center_pos = new_pos;
        transform.position = new_pos.extend(0.);
    }
}

fn random_table_position(canteen: &CanteenModel, rng: &mut Prng) -> Vec2 {
    // TODO: better placement logic
    vec2(
        rng.random_range(1.0..(canteen.width - 1.0)),
        rng.random_range(1.0..(canteen.windows_y - 1.0)),
    )
}

/// Open a new window with random service type
fn apply_open_window(
    _event: On<DispatchManagement<OpenWindowModel>>,
    mut commands: Commands,
    window_query: Query<(Entity, &Window)>,
    registry: Res<GameModelRegistryRes>,
    canteen: Res<Canteen>,
    mut rng: ResMut<WorldRng>,
) {
    let available_slots = (0..canteen.model.windows.len())
        .filter(|&i| !window_query.iter().any(|(_, w)| w.slot_index == i))
        .collect::<Vec<_>>();

    // Open a new window with random service type
    let service_model = registry
        .window_services
        .iter()
        .choose(&mut rng)
        .expect("No window models in registry")
        .clone();
    let service_handle = registry
        .window_services
        .get_handle_by_id(&service_model.id)
        .expect("Service model handle not found");

    if let Some(&slot_index) = available_slots.choose(&mut rng) {
        // Now only spawn the components needed for persistence
        let _window_entity = commands
            .spawn((Window {
                service_template: service_handle,
                slot_index,
                location: XSegment::new(0., 0., 0.), // does not matter
                disabled: false,
            },))
            .id();

        // Spawn dishes for this window
        // TODO: assign dishes
        /* for assignment in dish_assignments {
            let slot_index = assignment.slot_index;
            let Some(slot_rect) = layout.dish_slots.get(slot_index) else {
                continue;
            };

            let dish_handle = registry
                .dishes
                .get_handle_by_id(&assignment.dish_id)
                .expect("Dish not found in registry");
            let dish_model = registry.dishes.get(dish_handle);

            commands.spawn((
                Dish {
                    assignment: assignment.clone(),
                    state: DishRuntimeState {
                        current_quantity: DEFAULT_DISH_QUANTITY,
                        current_quality: DEFAULT_DISH_QUALITY,
                        contamination_level: DEFAULT_DISH_CONTAMINATION,
                        // last_restocked: DEFAULT_DISH_LAST_RESTOCKED_S,
                        // service_count: 0,
                    },
                },
                ServedAtWindow(window_entity),
            ));
        } */
    }
}

fn apply_close_window(
    _event: On<DispatchManagement<CloseWindowModel>>,
    mut commands: Commands,
    mut window_query: Query<Entity>,
    mut rng: ResMut<WorldRng>,
) {
    let Some(window) = window_query.iter_mut().choose(&mut rng) else {
        return;
    };

    commands.entity(window).despawn();
}

fn apply_change_window_service(_event: On<DispatchManagement<ChangeWindowServiceModel>>) {
    // TODO
}
