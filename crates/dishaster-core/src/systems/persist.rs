use crate::systems::prelude::*;

/// Sync diner long-term memory from active entities back to the persistent pool
///
/// This must run before persist_system to ensure memory changes during gameplay
/// are saved. It updates DinerProfile.long_term_memory for each active diner.
pub fn sync_diner_memory_system(
    diner_query: Query<(&Diner, &DinerLongTermMemory)>,
    mut diner_pool: ResMut<ResWrapper<DinerPool>>,
) {
    let mut profiles = diner_pool
        .profiles
        .iter_mut()
        .map(|profile| (profile.id, profile))
        .collect::<FxHashMap<_, _>>();
    for (diner, ltm) in diner_query.iter() {
        // Find the matching profile in the pool by ID
        if let Some(profile) = profiles.get_mut(&diner.id) {
            // Update the profile's long-term memory with current component state
            profile.long_term_memory = ltm.clone_inner();
        }
    }
}

pub fn persist_system(
    window_query: Query<(&Window, Option<&WindowDishes>)>,
    dish_query: Query<&Dish>,
    table_query: Query<&DiningTable>,
    dispenser_query: Query<&Dispenser>,
    collector_query: Query<&DishCollector>,
    day_status: Res<DayStatus>,
    daily_stats: Res<DailyStats>,
    reputation: Res<ReputationStateRes>,
    diner_pool: Res<ResWrapper<DinerPool>>,
    perma_effects: Res<PermanentEffectsRes>,
    registry: Res<GameModelRegistryRes>,
    level: Res<ResWrapper<LevelSetupState>>,
) -> SimProfile {
    let window_configurations = window_query
        .iter()
        .map(|(window, dishes)| {
            let service = registry.window_services.get(window.service_template);

            let price_override = dishes
                .map(|dishes| {
                    dishes
                        .iter()
                        .filter_map(|dish_entity| {
                            let dish = dish_query.get(dish_entity).ok()?;
                            let price = dish.pricing;
                            if service
                                .dish_options
                                .iter()
                                .any(|opt| opt.dish_id == dish.model_id && opt.pricing == price)
                            {
                                return None;
                            }
                            Some((dish.model_id.clone(), price))
                        })
                        .collect()
                })
                .unwrap_or_default();

            WindowConfiguration {
                slot_index: window.slot_index,
                service: service.id.clone(),
                is_disabled: window.disabled,
                price_override,
            }
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

    // Prepare daily stats from the current day
    let day_stats = DayStats {
        day: day_status.current_day,
        total_visits: daily_stats.total_visits,
        completed_diners: daily_stats.completed_diners,
        revenue: daily_stats.total_revenue,
        consumption_kg: daily_stats.total_consumption_kg,
        serving_times: daily_stats.serving_times.clone(),
        dining_times: daily_stats.dining_times.clone(),
        diner_orders: daily_stats.diner_orders.clone(),
    };

    SimProfile {
        level_id: level.level_id.clone(),
        current_day: day_status.current_day,
        reputation: ReputationProfile {
            reputation: reputation.reputation,
            fsri: reputation.fsri,
            food_quality: reputation.food_quality,
        },
        rng_seed: day_status.seed,
        window_configurations,
        placement,
        diner_profiles,
        permanent_effects: perma_effects.clone_inner(),
        day_stats,
    }
}
