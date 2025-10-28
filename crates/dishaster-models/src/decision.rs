/// Configuration for decision-making parameters
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
