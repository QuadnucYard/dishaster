//! Dish ordering decision system
//!
//! Implements realistic ordering behavior where diners select dishes based on:
//! - Hunger level and appetite
//! - Budget constraints
//! - Dish preferences (from LTM and STM intentions)
//! - Dish quality and pricing
//! - Variety seeking (avoid ordering same dish multiple times)

use super::prelude::*;
use crate::utils::sigmoid;

/// Estimate dish price for decision-making before ordering
///
/// For ByWeight dishes, uses the mean weight from the weight distribution.
/// This gives a more accurate estimate than a hardcoded value.
fn estimate_dish_price(pricing: &PricingMethod, characteristics: &DishCharacteristics) -> f32 {
    match pricing {
        PricingMethod::PerPortion(price) => *price,
        PricingMethod::ByWeight(price_per_kg) => price_per_kg * characteristics.weight_distrib.mean,
    }
}

/// Parameters for ordering decisions
#[derive(Debug, Clone, Resource)]
pub struct OrderingConfig {
    /// Tolerance for "close enough" to desired satiation (fraction of diner's max)
    /// When sat_needed <= tolerance × max_satiation, stop ordering
    pub satiation_tolerance: f32,
    /// Maximum number of different dishes one person can order
    pub max_dishes_per_order: usize,
    /// Weight for taste/preference in scoring (0..1)
    pub taste_weight: f32,
    /// Weight for quality in scoring (0..1)
    pub quality_weight: f32,
    /// Variety penalty factor (penalizes repeated dishes)
    pub variety_beta: f32,
    /// Sigmoid steepness for taste score
    pub sigmoid_k: f32,
    /// Maximum budget overspend factor (e.g., 1.2 = can spend up to 120% of budget)
    pub max_budget_overspend: f32,
    /// Base probability of accepting over-budget dish (0..1)
    pub overspend_base_prob: f32,
}

impl Default for OrderingConfig {
    fn default() -> Self {
        Self {
            satiation_tolerance: 0.05, // Stop when within 5% of desired satiation
            max_dishes_per_order: 4,
            taste_weight: 0.6,
            quality_weight: 0.4,
            variety_beta: 0.6,
            sigmoid_k: 2.0,
            max_budget_overspend: 1.3, // Can exceed budget by up to 30%
            overspend_base_prob: 0.3,  // 30% base chance to accept overspend
        }
    }
}

/// Decide what dishes to order at a window
///
/// Returns a list of dish IDs to order, in order of preference.
/// Uses a greedy algorithm that balances hunger, budget, preferences, and variety.
pub fn decide_order(
    window_dishes: &WindowDishes,
    dish_query: &Query<&Dish>,
    personality: &Personality,
    psych_state: &PsychState,
    dining_profile: &DiningProfile,
    ltm: &LongTermMemory,
    stm: &mut ShortTermMemory,
    registry: &GameModelRegistry,
    config: &OrderingConfig,
    meal_budget: f32,
    rng: &mut impl Rng,
) -> Vec<ServiceRequest> {
    // Use pre-calculated meal budget from spawn time
    let budget = meal_budget;

    // Calculate desired satiation based on hunger and individual capacity
    let desired_sat = psych_state.hunger * dining_profile.max_satiation;

    let mut sat_needed = desired_sat;
    let mut budget_left = budget;
    let mut orders = Vec::new();
    let mut variety_counts = FxHashMap::default();

    // Build list of available dishes with their info
    let mut candidates: Vec<_> = window_dishes
        .collection()
        .iter()
        .filter_map(|&dish_entity| {
            let dish = dish_query.get(dish_entity).ok()?;
            let dish_model = registry.dishes.get_by_id(&dish.model_id)?;

            // Estimate satiation contribution (use base_price as proxy for portion size)
            let sat_contribution = estimate_satiation(&dish_model.characteristics);

            Some((dish_entity, dish, dish_model, sat_contribution))
        })
        .collect();

    // Greedy selection loop
    for _ in 0..config.max_dishes_per_order {
        if sat_needed <= config.satiation_tolerance * dining_profile.max_satiation {
            break; // Close enough to satisfied (relative to diner's capacity)
        }

        if candidates.is_empty() {
            break; // No more options
        }

        // Score each candidate
        let mut best_idx: Option<usize> = None;
        let mut best_score = f32::NEG_INFINITY;

        for (i, (_, dish, dish_model, sat_contribution)) in candidates.iter().enumerate() {
            // Estimate price using weight distribution mean for ByWeight dishes
            let price = estimate_dish_price(&dish.pricing, &dish_model.characteristics);

            // Check if dish is within affordable range (including potential overspend)
            let max_acceptable_price = budget_left * config.max_budget_overspend;
            if price > max_acceptable_price + 1e-3 {
                continue; // Too expensive even with overspending
            }

            // Compute ordering utility score
            let score = compute_ordering_score(
                &dish_model.characteristics,
                &dish.model_id,
                price,
                dish.state.current_quality,
                *sat_contribution,
                personality,
                ltm,
                stm,
                &variety_counts,
                config,
            );

            // Add small deterministic noise for tie-breaking
            let dish_id_str = format!("{}", dish.model_id);
            let tie_noise = deterministic_noise_f32(rng.next_u64(), &dish_id_str);
            let final_score = score + tie_noise * 0.0001;

            if final_score > best_score {
                best_score = final_score;
                best_idx = Some(i);
            }
        }

        // Select best candidate
        let Some(idx) = best_idx else {
            break; // All remaining dishes are unaffordable
        };

        let (_dish_entity, dish, dish_model, sat_contribution) = candidates.remove(idx);
        let price = estimate_dish_price(&dish.pricing, &dish_model.characteristics);

        // Check if we need to overspend for this dish
        let overspend_amount = (price - budget_left).max(0.0);
        if overspend_amount > 0.0 {
            // Calculate probability of accepting overspend
            // Higher hunger and lower frugality increase acceptance
            let hunger_factor = psych_state.hunger; // 0..1
            let frugality_penalty = personality.frugality; // 0..1, higher = less likely to overspend
            let overspend_prob = config.overspend_base_prob
                * (1.0 + hunger_factor)
                * (1.0 - frugality_penalty * 0.5);

            // Roll dice to see if diner accepts overspending
            let accept_roll = rng.random_range(0.0..1.0);
            if accept_roll > overspend_prob {
                // Rejected due to budget concerns
                continue;
            }
        }

        // Add to order
        orders.push(ServiceRequest {
            dish_id: dish.model_id.clone(),
            dish_name: format!("{}", dish_model.id).into(),
            base_service_time: dish_model.characteristics.serving_time,
        });

        // Update state
        budget_left -= price;
        sat_needed -= sat_contribution;
        *variety_counts.entry(dish.model_id.clone()).or_insert(0) += 1;

        // Record in STM
        stm.current_order.push(dish.model_id.clone());
    }

    orders
}

/// Estimate how much satiation a dish provides based on its weight and filling properties
fn estimate_satiation(chars: &DishCharacteristics) -> f32 {
    // Use mean weight × satiation per kg
    chars.weight_distrib.mean * chars.satiation_per_kg
}

/// Compute ordering utility score for a dish
fn compute_ordering_score(
    chars: &DishCharacteristics,
    dish_id: &ModelId,
    current_price: f32,
    quality: f32,
    sat_contribution: f32,
    personality: &Personality,
    ltm: &LongTermMemory,
    stm: &ShortTermMemory,
    variety_counts: &FxHashMap<ModelId, usize>,
    config: &OrderingConfig,
) -> f32 {
    // Compute taste preference score
    let taste_pref = compute_taste_preference(&chars.tags, dish_id, ltm, stm);
    let taste_score = sigmoid(config.sigmoid_k * (taste_pref + 0.5 * (quality - 0.5)));

    // Base utility combining taste and quality
    let utility = config.taste_weight * taste_score + config.quality_weight * quality;

    // Compute efficiency metrics
    let price_efficiency = utility / current_price.max(0.1);
    let sat_efficiency = utility / sat_contribution.max(1.0);

    // Blend based on frugality
    let alpha = personality.frugality;
    let mut metric = alpha * price_efficiency + (1.0 - alpha) * sat_efficiency;

    // Apply variety penalty
    let times_ordered = *variety_counts.get(dish_id).unwrap_or(&0);
    let variety_factor = 1.0 / (1.0 + config.variety_beta * times_ordered as f32);
    metric *= variety_factor;

    metric
}

/// Compute taste preference for a dish
fn compute_taste_preference(
    tags: &[EcoString],
    dish_id: &ModelId,
    ltm: &LongTermMemory,
    stm: &ShortTermMemory,
) -> f32 {
    // Check STM intentions first (stronger signal)
    let intention_bonus = stm.dish_intentions.get(dish_id).copied().unwrap_or(0.0);

    // Check tag preferences from LTM
    let mut tag_sum = 0.0;
    let mut tag_count = 0;
    for tag in tags {
        if let Some(&weight) = ltm.like_tags.get(tag) {
            tag_sum += weight;
            tag_count += 1;
        }
    }
    let tag_pref = if tag_count > 0 {
        tag_sum / tag_count as f32
    } else {
        0.0
    };

    // Check dish-specific memory
    let dish_pref = ltm
        .dish_experience
        .get(dish_id)
        .map(|mem| mem.avg_rating)
        .unwrap_or(0.0);

    // Combine: intention is strongest, then dish-specific, then tags
    intention_bonus * 0.5 + dish_pref * 0.3 + tag_pref * 0.2
}

/// Generate deterministic noise for tie-breaking
fn deterministic_noise_f32(seed: u64, key: &str) -> f32 {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    seed.hash(&mut hasher);
    key.hash(&mut hasher);
    let h = hasher.finish();

    // Map to [-1, 1]
    let v = (h % 100000) as f32 / 100000.0;
    (v - 0.5) * 2.0
}
