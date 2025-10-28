//! Diner decision-making system with realistic scoring logic.
//!
//! This module implements the enhanced decision system described in the design document,
//! using mathematical scoring functions to model diner preferences, psychological state,
//! and memory-based decision making.

use std::sync::Arc;

use super::prelude::*;

// ===================== Scoring Functions =====================

/// Compute overall dish score combining all factors
///
/// This is the main scoring function that combines taste, quality, price, novelty,
/// wait time, and mood effects into a single score for decision making.
fn compute_dish_score(
    dish_tags: &[EcoString],
    dish_id: &ModelId,
    current_price: f32,
    base_price: f32,
    quality: f32,
    estimated_wait: f32,
    personality: &Personality,
    psych_state: &PsychState,
    ltm: &LongTermMemory,
    config: &DecisionConfig,
) -> f32 {
    let weights = &config.weights;

    // Compute individual scores
    let taste = compute_taste_score(dish_tags, ltm, dish_id);
    let price = compute_price_score(current_price, base_price, personality.frugality);
    let quality_score = quality.clamp(0.0, 1.0);
    let novelty = compute_novelty(dish_id, ltm, personality.adventurous);

    // Base score (weighted sum)
    let base_score = weights.taste * taste
        + weights.quality * quality_score
        + weights.price * price
        + weights.novelty * novelty;

    // Apply wait time penalty
    let wait_mult = compute_wait_multiplier(
        estimated_wait,
        psych_state.patience_now,
        config.wait_penalty_gamma,
    );

    // Apply mood modifier (positive mood increases risk-taking, negative reduces it)
    let mood_mult = 1.0 + psych_state.mood * 0.2;

    base_score * wait_mult * mood_mult
}

/// Compute taste/preference score for a dish based on tags and memory
///
/// Combines tag preferences from long-term memory with past experience ratings.
/// Returns a score in range 0..1+ (can exceed 1 for highly preferred dishes).
fn compute_taste_score(dish_tags: &[EcoString], ltm: &LongTermMemory, dish_id: &ModelId) -> f32 {
    // Sum up tag preferences
    let mut tag_sum = 0.0;
    let mut tag_count = 0;

    for tag in dish_tags {
        if let Some(&weight) = ltm.like_tags.get(tag) {
            tag_sum += weight;
            tag_count += 1;
        }
    }

    // Average tag preference (normalized to -1..1)
    let avg_tag_pref = if tag_count > 0 {
        tag_sum / tag_count as f32
    } else {
        0.0
    };

    // Apply sigmoid to map to 0..1
    let tag_score = sigmoid(avg_tag_pref, 1.0);

    // Add memory-based rating if dish has been tried before
    let mem_rating = ltm
        .dish_experience
        .get(dish_id)
        .map(|m| m.avg_rating)
        .unwrap_or(0.0);

    // Combine: base from tags, bonus from positive memories
    tag_score + 0.5 * mem_rating.max(0.0)
}

/// Compute price attractiveness score
///
/// Penalizes overpriced dishes, slightly rewards underpriced ones.
/// Frugality increases price sensitivity.
fn compute_price_score(current_price: f32, base_price: f32, frugality: f32) -> f32 {
    let price_ratio = current_price / base_price.max(0.01);

    // Linear penalty for overpricing, small bonus for underpricing
    // Adjusted by frugality (higher frugality = more price sensitive)
    let sensitivity = 0.6 * (1.0 + frugality);
    let score = 1.2 - sensitivity * (price_ratio - 1.0);

    score.clamp(0.2, 1.2)
}

/// Compute novelty bonus based on how many times dish has been eaten
///
/// Rewards adventurous diners for trying new dishes.
fn compute_novelty(dish_id: &ModelId, ltm: &LongTermMemory, adventurous: f32) -> f32 {
    let times_eaten = ltm
        .dish_experience
        .get(dish_id)
        .map(|m| m.times_eaten)
        .unwrap_or(0);

    // Novelty decreases with repeated consumption
    let novelty = 1.0 - (times_eaten as f32 / 3.0).min(1.0);

    adventurous * novelty
}

/// Compute wait time penalty multiplier
///
/// Exponentially penalizes wait times that exceed patience threshold.
fn compute_wait_multiplier(estimated_wait: f32, patience_now: f32, gamma: f32) -> f32 {
    if estimated_wait <= patience_now {
        return 1.0;
    }

    let excess_ratio = (estimated_wait - patience_now) / patience_now.max(1.0);
    let penalty = gamma * excess_ratio;

    (-penalty).exp()
}

// ===================== Psychological State Updates =====================

/// Update patience based on personality, mood, and trust
pub fn update_patience(personality: &Personality, psych_state: &mut PsychState) {
    psych_state.patience_now = personality.patience_base
        * (1.0 + 0.3 * psych_state.trust)
        * (1.0 + 0.25 * psych_state.mood);
}

/// Apply mood decay toward baseline (exponential decay)
pub fn apply_mood_decay(psych_state: &mut PsychState, delta_time: f32, tau_mood: f32) {
    let decay_rate = delta_time / tau_mood;
    psych_state.mood *= (1.0 - decay_rate).max(0.0);
}

/// Update psychological state after abandoning a queue
pub fn handle_abandon_penalty(
    psych_state: &mut PsychState,
    ltm: &mut LongTermMemory,
    estimated_wait: f32,
    patience_now: f32,
    config: &DecisionConfig,
) {
    let excess_ratio = (estimated_wait - patience_now) / patience_now.max(1.0);
    let mood_penalty = config.abandon_mood_penalty * excess_ratio.max(0.0);

    psych_state.mood = (psych_state.mood - mood_penalty).max(-1.0);
    ltm.overall_like = (ltm.overall_like - 0.02).max(0.0);
}

// ===================== Satisfaction and Memory Updates =====================

/// Configuration for satisfaction computation
#[derive(Debug, Clone)]
pub struct SatisfactionWeights {
    /// Weight for taste component
    pub taste: f32,
    /// Weight for quality component
    pub quality: f32,
    /// Weight for price pain (penalty)
    pub price: f32,
    /// Weight for hunger satisfaction bonus
    pub hunger: f32,
}

impl Default for SatisfactionWeights {
    fn default() -> Self {
        Self {
            taste: 0.4,
            quality: 0.3,
            price: 0.2,
            hunger: 0.1,
        }
    }
}

/// Compute satisfaction from eating a dish
///
/// Returns a value in range -1..1 representing overall satisfaction.
/// Positive values increase mood and ratings, negative values decrease them.
pub fn compute_satisfaction(
    dish_tags: &[EcoString],
    dish_id: &ModelId,
    current_price: f32,
    base_price: f32,
    quality: f32,
    contamination_level: f32,
    hunger_before: f32,
    ltm: &LongTermMemory,
    weights: &SatisfactionWeights,
) -> f32 {
    // Taste component (based on preferences)
    let taste_component = compute_taste_score(dish_tags, ltm, dish_id);

    // Quality component
    let quality_component = quality.clamp(0.0, 1.0);

    // Price pain (positive if overpriced, negative if good deal)
    let price_ratio = current_price / base_price.max(0.01);
    let price_pain = price_ratio - 1.0;

    // Hunger satisfaction (more hungry = more satisfied when eating)
    let hunger_factor = hunger_before;

    // Safety penalty (contamination is very bad)
    let safety_penalty = if contamination_level > 0.1 {
        -2.0 * contamination_level // Severe penalty
    } else {
        0.0
    };

    // Combine components
    let satisfaction = weights.taste * taste_component + weights.quality * quality_component
        - weights.price * price_pain
        + weights.hunger * hunger_factor
        + safety_penalty;

    // Clamp to -1..1
    satisfaction.clamp(-1.0, 1.0)
}

/// Update diner's psychological state and memory after eating
///
/// This is the main function to call after a diner finishes eating.
pub fn update_after_eating(
    dish_tags: &[EcoString],
    dish_id: &ModelId,
    current_price: f32,
    base_price: f32,
    quality: f32,
    contamination_level: f32,
    current_time: f32,
    psych_state: &mut PsychState,
    ltm: &mut LongTermMemory,
    weights: &SatisfactionWeights,
) {
    let hunger_before = psych_state.hunger;

    // Compute satisfaction
    let satisfaction = compute_satisfaction(
        dish_tags,
        dish_id,
        current_price,
        base_price,
        quality,
        contamination_level,
        hunger_before,
        ltm,
        weights,
    );

    // Update mood (with scaling factor)
    let mood_gain = satisfaction * 0.3;
    psych_state.mood = (psych_state.mood + mood_gain).clamp(-1.0, 1.0);

    // Update hunger (eating reduces hunger)
    psych_state.hunger = (psych_state.hunger - 0.7).max(0.0);

    // Update dish memory
    let dish_mem = ltm.dish_experience.entry(dish_id.clone()).or_default();
    dish_mem.times_eaten += 1;
    let n = dish_mem.times_eaten as f32;
    dish_mem.avg_rating = (dish_mem.avg_rating * (n - 1.0) + satisfaction) / n;
    dish_mem.last_eaten = Some(current_time);

    // Update overall canteen satisfaction (exponential smoothing)
    let alpha = 0.1; // Smoothing factor
    let satisfaction_01 = (satisfaction + 1.0) / 2.0; // Map -1..1 to 0..1
    ltm.overall_like = ltm.overall_like * (1.0 - alpha) + satisfaction_01 * alpha;

    // Handle contamination specially (trust penalty)
    if contamination_level > 0.1 {
        psych_state.trust = (psych_state.trust - 0.3 * contamination_level).max(0.0);
        psych_state.mood = (psych_state.mood - 0.5).max(-1.0);
    }
}

// ===================== Utility Functions =====================

/// Sigmoid function for mapping unbounded values to 0..1
fn sigmoid(x: f32, k: f32) -> f32 {
    1.0 / (1.0 + (-k * x).exp())
}

/// Softmax sampling from scores
///
/// Returns index of selected item based on softmax probabilities.
fn sample_softmax(scores: &[f32], temperature: f32, rng: &mut impl Rng) -> Option<usize> {
    if scores.is_empty() {
        return None;
    }

    // Apply temperature and find max for numerical stability
    let max_score = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    // Compute exp(score/T - max/T) for stability
    let exp_scores: Vec<f32> = scores
        .iter()
        .map(|&s| ((s - max_score) / temperature).exp())
        .collect();

    let sum: f32 = exp_scores.iter().sum();

    if sum <= 0.0 {
        return None;
    }

    // Sample using cumulative distribution
    let threshold = rng.random_range(0.0..sum);
    let mut cumulative = 0.0;

    for (i, &exp_score) in exp_scores.iter().enumerate() {
        cumulative += exp_score;
        if cumulative >= threshold {
            return Some(i);
        }
    }

    // Fallback (should not reach here)
    Some(scores.len() - 1)
}

// ===================== Window Selection Logic =====================

/// Information about a window candidate for decision-making
#[derive(Debug, Clone)]
pub struct WindowCandidate {
    /// Entity ID of the window
    pub window_entity: Entity,
    /// Overall score for this window
    pub score: f32,
}

/// Evaluate a window and return its score based on available dishes
///
/// This function looks at all dishes in a window and computes an aggregate score.
/// For now, we use the maximum dish score as the window score (best dish determines appeal).
pub fn evaluate_window(
    window_entity: Entity,
    dishes: &[dishaster_models::ActiveDish],
    queue_length: usize,
    avg_service_time: f32,
    personality: &Personality,
    psych_state: &PsychState,
    ltm: &LongTermMemory,
    registry: &Arc<GameModelRegistry>,
    config: &DecisionConfig,
) -> Option<WindowCandidate> {
    if dishes.is_empty() {
        return None;
    }

    let estimated_wait = queue_length as f32 * avg_service_time;

    // Score each dish in the window
    let mut dish_scores = Vec::new();

    for active_dish in dishes {
        let dish_id = &active_dish.assignment.dish_id;

        // Get dish model for tags and base price
        let Some(dish_model) = registry.dishes.get_by_id(dish_id) else {
            continue;
        };

        // Use tags from model if available
        let dish_tags = &dish_model.characteristics.tags;

        // Get current price
        let current_price = match active_dish.assignment.pricing.method {
            dishaster_models::PricingMethod::PerPortion(price) => price,
            dishaster_models::PricingMethod::ByWeight(price_per_kg) => price_per_kg * 0.2, // Assume 200g portion
        };

        // Use base price from model, or estimate if not set
        let base_price = if dish_model.characteristics.base_price > 0.0 {
            dish_model.characteristics.base_price
        } else {
            current_price * 0.9 // Fallback: assume slight markup
        };

        let quality = active_dish.state.current_quality;

        let score = compute_dish_score(
            dish_tags,
            dish_id,
            current_price,
            base_price,
            quality,
            estimated_wait,
            personality,
            psych_state,
            ltm,
            config,
        );

        dish_scores.push(score);
    } // Use max score as window score (best dish determines appeal)
    let window_score = dish_scores.into_iter().fold(0.0f32, f32::max);

    Some(WindowCandidate {
        window_entity,
        score: window_score,
    })
}

/// Select a window from candidates using softmax sampling
///
/// Returns the chosen window entity, or None if no suitable windows found.
pub fn select_window_from_candidates(
    candidates: &[WindowCandidate],
    config: &DecisionConfig,
    rng: &mut impl Rng,
) -> Option<Entity> {
    if candidates.is_empty() {
        return None;
    }

    let scores: Vec<f32> = candidates.iter().map(|c| c.score).collect();

    let selected_idx = sample_softmax(&scores, config.temperature, rng)?;

    Some(candidates[selected_idx].window_entity)
}

// ===================== Tests =====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_score() {
        // Same price should give good score
        let score = compute_price_score(10.0, 10.0, 0.5);
        assert!(score > 1.0);

        // Overpriced should penalize
        let overpriced = compute_price_score(15.0, 10.0, 0.5);
        assert!(overpriced < score);

        // Higher frugality should increase penalty
        let more_frugal = compute_price_score(15.0, 10.0, 0.9);
        assert!(more_frugal < overpriced);
    }

    #[test]
    fn test_wait_multiplier() {
        // Within patience should not penalize
        assert!((compute_wait_multiplier(30.0, 60.0, 3.0) - 1.0).abs() < 0.01);

        // Exceeding patience should penalize
        assert!(compute_wait_multiplier(120.0, 60.0, 3.0) < 0.5);
    }
}
