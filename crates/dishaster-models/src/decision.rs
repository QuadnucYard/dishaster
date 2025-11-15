use crate::prelude::*;

/// Configuration for decision-making parameters
///
/// Controls how diners choose which window or canteen to visit based on
/// various factors like taste, quality, price, and novelty.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionConfig {
    /// Scoring weights
    pub weights: ScoringWeights,
    /// Temperature for softmax sampling (lower = greedier)
    pub temperature: f32,
    /// Wait penalty gamma factor
    pub wait_penalty_gamma: f32,
    /// Price injustice threshold (fraction above base price)
    pub price_injustice_threshold: f32,
    /// Abandon penalty mood impact
    pub abandon_mood_penalty: f32,
}

impl Default for DecisionConfig {
    fn default() -> Self {
        Self {
            weights: ScoringWeights::default(),
            temperature: 1.0,
            wait_penalty_gamma: 3.0,
            price_injustice_threshold: 0.25,
            abandon_mood_penalty: 0.2,
        }
    }
}

/// Tunable weights for combining different scoring factors
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoringWeights {
    /// Weight for taste/preference score (0..1)
    pub taste: f32,
    /// Weight for quality score (0..1)
    pub quality: f32,
    /// Weight for price attractiveness (0..1)
    pub price: f32,
    /// Weight for novelty bonus (0..1)
    pub novelty: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            taste: 0.45,
            quality: 0.25,
            price: 0.2,
            novelty: 0.1,
        }
    }
}

/// Configuration for ordering decisions
///
/// Controls how diners select which dishes to order based on hunger,
/// budget, preferences, and variety-seeking behavior.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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
