use super::prelude::*;

/// Update dish contamination levels based on FSRI and dish risk level
///
/// Contamination accumulates over time based on:
/// - FSRI (Food Safety Risk Index) - higher FSRI means faster contamination
/// - Dish risk_level - inherent contamination risk of the dish type
///
/// This creates gameplay where poor food safety management leads to actual
/// contamination that diners can detect, triggering hygiene feedback.
pub fn update_dish_contamination(
    mut dish_query: Query<(&mut Dish, &ServedAtWindow)>,
    reputation: Res<ReputationStateRes>,
    registry: Res<GameModelRegistryRes>,
    time: Res<Time>,
) {
    let dt = time.tick_duration as f32;

    // Base contamination rate scaled by FSRI (0-100)
    // At FSRI=0: no contamination
    // At FSRI=50: moderate contamination rate
    // At FSRI=100: high contamination rate
    let fsri_factor = reputation.fsri / 100.0;

    for (mut dish, _) in dish_query.iter_mut() {
        let Some(dish_model) = registry.dishes.get_by_id(&dish.model_id) else {
            continue;
        };

        // Each dish has its own risk_level (0-1)
        let risk_level = dish_model.characteristics.risk_level;

        // Contamination rate: fsri_factor * risk_level * base_rate
        // With FSRI=10 (default) and risk=0.1 (typical):
        //   - fsri_factor = 0.1
        //   - contamination_rate = 0.1 * 0.1 * 0.0003 = 0.000003/s
        //   - In 1 hour (3600s): 0.0108
        //   - Dishes need ~55 minutes to reach threshold 0.01
        // Target: ~3-10 hygiene feedback per day with FSRI=10
        // Higher FSRI dramatically increases contamination risk
        let base_rate = 0.0004; // Moderate risk to make FSRI management important
        let contamination_rate = fsri_factor * risk_level * base_rate;

        // Apply contamination increase
        let old_contamination = dish.state.contamination_level;
        dish.state.contamination_level += contamination_rate * dt;

        // Cap at reasonable maximum (1.0)
        dish.state.contamination_level = dish.state.contamination_level.min(1.0);

        // Log significant contamination increases for debugging
        if old_contamination < 0.01 && dish.state.contamination_level >= 0.01 {
            log::debug!(
                target: "contamination",
                "Dish {:?} reached hygiene threshold: {:.4} (FSRI={:.1}, risk={:.2})",
                dish.model_id,
                dish.state.contamination_level,
                reputation.fsri,
                risk_level
            );
        }
    }
}
