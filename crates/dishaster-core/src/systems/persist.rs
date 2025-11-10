use crate::systems::prelude::*;

pub fn persist_system(
    table_query: Query<&DiningTable>,
    dispenser_query: Query<&Dispenser>,
    collector_query: Query<&DishCollector>,
    level: Res<ResWrapper<LevelSetupState>>,
    diner_pool: Res<ResWrapper<DinerPool>>,
) -> SimProfile {
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
                model: disp.model_id.clone(),
            })
            .collect(),
        chopstick_dispensers: dispenser_query
            .iter()
            .filter(|it| it.dispenser_type == DispenserType::Chopstick)
            .map(|disp| Placement {
                center_pos: disp.center_pos,
                model: disp.model_id.clone(),
            })
            .collect(),
        collectors: collector_query
            .iter()
            .map(|collector| Placement {
                center_pos: collector.center_pos,
                model: collector.model_id.clone(),
            })
            .collect(),
    };

    let diner_profiles = diner_pool.profiles.clone();

    SimProfile {
        current_day: level.day,
        rng_seed: level.seed,
        window_configurations: level.canteen.window_configurations.clone(),
        placement,
        diner_profiles,
    }
}
