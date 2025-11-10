use crate::systems::prelude::*;

pub fn persist_system(
    window_query: Query<(&Window, &WindowDishes)>,
    dish_query: Query<&Dish>,
    table_query: Query<&DiningTable>,
    dispenser_query: Query<&Dispenser>,
    collector_query: Query<&DishCollector>,
    day_status: Res<DayStatus>,
    diner_pool: Res<ResWrapper<DinerPool>>,
    registry: Res<GameModelRegistryRes>,
) -> SimProfile {
    let window_configurations = window_query
        .iter()
        .map(|(window, dishes)| WindowConfiguration {
            slot_index: window.slot_index,
            service_template: registry
                .window_services
                .get_id(window.service_template)
                .clone(),
            is_enabled: window.enabled,
            dish_assignments: dishes
                .iter()
                .map(|dish_entity| {
                    dish_query
                        .get(dish_entity)
                        .expect("dish for window")
                        .assignment
                        .clone()
                })
                .collect(),
        })
        .collect();

    let placement = CanteenPlacements {
        tables: table_query
            .iter()
            .map(|table| Placement {
                center_pos: table.center_pos,
                model: table.model_id.clone(),
            })
            .collect(),
        tray_dispensers: dispenser_query
            .iter()
            .filter(|it| it.dispenser_type == DispenserType::Tray)
            .map(|disp| Placement {
                center_pos: disp.center_pos,
                model: registry.dispensers.get_id(disp.model).clone(),
            })
            .collect(),
        chopstick_dispensers: dispenser_query
            .iter()
            .filter(|it| it.dispenser_type == DispenserType::Chopstick)
            .map(|disp| Placement {
                center_pos: disp.center_pos,
                model: registry.dispensers.get_id(disp.model).clone(),
            })
            .collect(),
        collectors: collector_query
            .iter()
            .map(|collector| Placement {
                center_pos: collector.center_pos,
                model: registry.collectors.get_id(collector.model).clone(),
            })
            .collect(),
    };

    let diner_profiles = diner_pool.profiles.clone();

    SimProfile {
        current_day: day_status.current_day,
        rng_seed: day_status.seed,
        window_configurations,
        placement,
        diner_profiles,
    }
}
