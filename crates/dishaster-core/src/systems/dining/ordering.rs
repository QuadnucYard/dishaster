//! Dish ordering decision system
//!
//! Implements realistic ordering behavior where diners select dishes based on:
//! - Hunger level and appetite
//! - Budget constraints
//! - Dish preferences (from LTM and STM intentions)
//! - Dish quality and pricing
//! - Variety seeking (avoid ordering same dish multiple times)

use crate::{
    systems::{choose_feedback, feedbacks, prelude::*},
    utils::sigmoid,
};

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

/// Create session at front of queue using validated tentative order
pub fn handle_create_session_from_tentative_order(
    commands: &mut Commands,
    entity: Entity,
    goal: &mut DinerGoalState,
    diner_state: &mut DinerState,
    targets: &mut DinerTargets,
    psych_state: &mut PsychState,
    rng: &mut EntityRng,
    queue_member: &QueueMember,
    window_dishes_query: &Query<&WindowDishes>,
    dish_query: &Query<&Dish>,
    lane_query: &Query<(&QueueLane, &QueueLaneMembers)>,
    time: &Time,
    feedback_messages: &mut MessageWriter<FeedbackMessage>,
) {
    let Ok((lane, _)) = lane_query.get(queue_member.lane) else {
        return;
    };
    let window_entity = lane.owner;

    // Verify window still has dishes
    let Some(window_dishes) = window_dishes_query.get(window_entity).ok() else {
        log::warn!(
            target: "diner",
            "diner {:?} at front of queue but window {:?} has no dishes",
            entity,
            window_entity
        );
        commands.entity(entity).remove::<QueueMember>();
        goal.update(DinerGoal::Leave);
        return;
    };

    // Validate tentative order exists
    if targets.tentative_order.is_empty() {
        log::warn!(
            target: "diner",
            "diner {:?} at front of queue but has no tentative order, leaving",
            entity
        );
        apply_no_order_penalties(entity, psych_state, rng, feedback_messages);
        commands.entity(entity).remove::<QueueMember>();
        goal.update(DinerGoal::Leave);
        return;
    }

    // Validate dishes are still available
    let planned_order =
        validate_tentative_order(window_dishes, dish_query, &targets.tentative_order);

    if planned_order.is_empty() {
        log::warn!(
            target: "diner",
            "diner {:?} tentative order no longer valid (all dishes unavailable), leaving",
            entity
        );
        apply_dishes_unavailable_penalties(entity, psych_state, rng, feedback_messages);
        commands.entity(entity).remove::<QueueMember>();
        goal.update(DinerGoal::Leave);
        return;
    }

    // Log order changes
    if planned_order.len() < targets.tentative_order.len() {
        log::debug!(
            target: "diner",
            "diner {:?} order reduced from {} to {} dishes due to availability",
            entity,
            targets.tentative_order.len(),
            planned_order.len()
        );
    }

    // Create session
    diner_state.total_spent = 0.0;
    log::debug!(
        target: "diner",
        "diner {:?} confirming tentative order with {} dishes",
        entity,
        planned_order.len()
    );

    let mut session = ServiceSession::new(window_entity, queue_member.lane, time.current_time);
    session.planned_order = planned_order;
    commands.entity(entity).insert(session);
    targets.tentative_order.clear();
    goal.update(DinerGoal::GetServed);
}

/// Allow diners waiting in queue to re-evaluate their order
pub fn handle_queue_re_evaluation(
    commands: &mut Commands,
    entity: Entity,
    goal: &mut DinerGoalState,
    diner_state: &mut DinerState,
    targets: &mut DinerTargets,
    personality: &Personality,
    psych_state: &mut PsychState,
    dining_profile: &DiningProfile,
    ltm: &LongTermMemory,
    stm: &mut ShortTermMemory,
    rng: &mut EntityRng,
    queue_member: &QueueMember,
    window_dishes_query: &Query<&WindowDishes>,
    dish_query: &Query<&Dish>,
    registry: &GameModelRegistry,
    ordering_config: &OrderingConfig,
    feedback_messages: &mut MessageWriter<FeedbackMessage>,
) {
    let window_entity = targets.chosen_window.unwrap();

    // Check if window still has dishes
    let Some(window_dishes) = window_dishes_query.get(window_entity).ok() else {
        return;
    };

    log::debug!(
        target: "diner",
        "diner {:?} re-evaluating order while waiting (ranking={})",
        entity,
        queue_member.ranking
    );

    // Re-evaluate order with current psychological state
    let new_order = decide_order(
        window_dishes,
        dish_query,
        personality,
        psych_state,
        dining_profile,
        ltm,
        stm,
        registry,
        ordering_config,
        diner_state.meal_budget,
        rng,
    );

    if new_order.is_empty() {
        log::info!(
            target: "diner",
            "diner {:?} abandoning queue after re-evaluation: no valid order",
            entity
        );
        apply_abandon_after_reevaluation_penalties(entity, psych_state, rng, feedback_messages);
        commands.entity(entity).remove::<QueueMember>();
        goal.update(DinerGoal::Leave);
        return;
    }

    // Update tentative order and reset cooldown
    targets.tentative_order = new_order;
    goal.reset_timer();
}

/// Validate tentative order against current window dishes
fn validate_tentative_order(
    window_dishes: &WindowDishes,
    dish_query: &Query<&Dish>,
    tentative_order: &[ServiceRequest],
) -> Vec<ServiceRequest> {
    let available_dish_ids: FxHashSet<_> = window_dishes
        .collection()
        .iter()
        .filter_map(|&entity| dish_query.get(entity).ok().map(|d| d.model_id.clone()))
        .collect();

    tentative_order
        .iter()
        .filter(|req| available_dish_ids.contains(&req.dish_id))
        .cloned()
        .collect()
}

/// Apply penalties when diner has no tentative order
fn apply_no_order_penalties(
    entity: Entity,
    psych_state: &mut PsychState,
    rng: &mut EntityRng,
    feedback_messages: &mut MessageWriter<FeedbackMessage>,
) {
    psych_state.mood = (psych_state.mood - 0.3).max(-1.0);
    psych_state.trust = (psych_state.trust - 0.2).max(0.0);

    feedback_messages.write(FeedbackMessage {
        entity,
        content: choose_feedback(rng, feedbacks::NO_APPEALING_DISH),
        trigger: Some(FeedbackTopic::Appeal),
    });
}

/// Apply penalties when dishes become unavailable
fn apply_dishes_unavailable_penalties(
    entity: Entity,
    psych_state: &mut PsychState,
    rng: &mut EntityRng,
    feedback_messages: &mut MessageWriter<FeedbackMessage>,
) {
    psych_state.mood = (psych_state.mood - 0.3).max(-1.0);
    psych_state.trust = (psych_state.trust - 0.15).max(0.0);

    feedback_messages.write(FeedbackMessage {
        entity,
        content: choose_feedback(rng, feedbacks::NO_APPEALING_DISH),
        trigger: Some(FeedbackTopic::Appeal),
    });
}

/// Apply penalties when abandoning after re-evaluation
fn apply_abandon_after_reevaluation_penalties(
    entity: Entity,
    psych_state: &mut PsychState,
    rng: &mut EntityRng,
    feedback_messages: &mut MessageWriter<FeedbackMessage>,
) {
    psych_state.mood = (psych_state.mood - 0.25).max(-1.0);
    psych_state.trust = (psych_state.trust - 0.15).max(0.0);

    feedback_messages.write(FeedbackMessage {
        entity,
        content: choose_feedback(rng, feedbacks::NO_APPEALING_DISH),
        trigger: Some(FeedbackTopic::Appeal),
    });
}
