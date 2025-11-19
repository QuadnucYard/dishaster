use enum_map::{Enum, EnumMap, enum_map};

use super::prelude::*;

/// Different feedback topics that can be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Enum)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackTopic {
    /// No dishes appealed to the diner
    Appeal,
    /// Queue too long / exceeded patience
    Queue,
    /// Missing tableware (tray or chopsticks)
    Tableware,
    /// Dish below expectation
    Quality,
    /// Pricing complaints
    Price,
    /// Food hygiene issues encountered
    Hygiene,
    /// Dish tastes bad
    Taste,
    /// Still hungry after meal
    Hunger,
    /// Positive feedback
    Praise,
    /// Special topic
    Crab,
}

/// Configuration thresholds for triggering feedback
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FeedbackThresholds {
    /// Minimum dish score required during observation (below triggers NoAppealingDish)
    pub min_appealing_score: f32,

    /// Minimum contamination level to trigger Contamination feedback
    pub contamination_threshold: f32,

    /// Maximum satisfaction score to trigger BadTaste feedback
    pub bad_taste_threshold: f32,

    /// Minimum hunger level after eating to trigger StillHungry feedback
    pub still_hungry_threshold: f32,

    /// Base tolerance for quality mismatch (modified by personality)
    pub base_quality_tolerance: f32,

    /// Weight for historical memory in expected quality calculation
    pub memory_weight: f32,

    /// Weight for dish base quality in expected quality calculation
    pub base_quality_weight: f32,

    /// Maximum price ratio (price/base_price) before triggering price complaint
    pub max_price_ratio: f32,

    /// Minimum satisfaction score to trigger praise feedback
    pub praise_threshold: f32,
}

impl Default for FeedbackThresholds {
    fn default() -> Self {
        Self {
            min_appealing_score: -0.3,
            contamination_threshold: 0.1,
            bad_taste_threshold: -0.3,
            still_hungry_threshold: 0.3,
            base_quality_tolerance: 0.2,
            memory_weight: 0.6,
            base_quality_weight: 0.4,
            max_price_ratio: 1.3,
            praise_threshold: 0.4,
        }
    }
}

/// Configuration for reputation system
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ReputationConfig {
    /// Base impact values for each feedback topic on reputation
    /// Positive values increase reputation, negative values decrease it
    pub base_impacts: EnumMap<FeedbackTopic, f32>,

    /// Probability of showing feedback bubble for each topic
    ///
    /// Controls visual feedback frequency. Lower values = fewer bubbles shown.
    /// Note: Reputation impact still calculated even if bubble not shown.
    pub display_probabilities: EnumMap<FeedbackTopic, f32>,

    /// Probability of applying reputation impact for each feedback topic
    ///
    /// Gates whether feedback affects reputation. This is separate from display probability.
    /// Lower values = more forgiving (some complaints ignored for reputation).
    pub impact_probabilities: EnumMap<FeedbackTopic, f32>,

    /// Thresholds for triggering different types of feedback
    pub feedback_thresholds: FeedbackThresholds,

    /// How much player response affects the impact (0..1)
    /// Higher means player responses have more effect
    pub response_factor: f32,

    /// Maximum absolute change from a single feedback event
    pub max_single_change: f32,

    /// Maximum absolute reputation change in one day
    pub max_daily_change: f32,

    /// FSRI incident probability multiplier
    pub fsri_incident_multiplier: f32,

    /// Maximum incident probability per day (clamped)
    pub max_incident_probability: f32,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        use FeedbackTopic::*;
        Self {
            base_impacts: enum_map! {
                Praise => 3.0,
                Appeal => -2.0,
                Queue => -3.0,
                Tableware => -2.5,
                Quality => -4.0,
                Price => -3.0,    // Pricing complaints
                Hygiene => -12.0, // Most severe
                Taste => -3.5,
                Hunger => -2.0,
                Crab => 0.0,
            },
            display_probabilities: enum_map! {
                Praise => 0.2,    // Show 20% of praise
                Appeal => 0.3,    // Show 30% of appeal issues
                Queue => 0.4,     // Show 40% of queue complaints
                Tableware => 0.3, // Show 30 of tableware issues
                Quality => 0.3,   // Show 30% of quality issues
                Price => 0.4,     // Show 40% of price complaints
                Hygiene => 1.0,   // Always show hygiene (critical)
                Taste => 0.3,     // Show 30% of taste complaints
                Hunger => 0.2,    // Show 20% of hunger complaints
                Crab => 0.0,
            },
            impact_probabilities: enum_map! {
                Praise => 1.0,    // Always apply positive feedback
                Appeal => 0.4,    // Apply 40% of appeal issues
                Queue => 0.6,     // Apply 60% of queue complaints
                Tableware => 0.5, // Apply 50% of tableware issues
                Quality => 0.4,   // Apply 40% of quality issues
                Price => 0.5,     // Apply 50% of price complaints
                Hygiene => 1.0,   // Always apply hygiene (critical)
                Taste => 0.4,     // Apply 40% of taste complaints
                Hunger => 0.3,    // Apply 30% of hunger complaints
                Crab => 1.0,
            },
            feedback_thresholds: FeedbackThresholds::default(),
            response_factor: 0.6,
            max_single_change: 8.0,
            max_daily_change: 12.0,
            fsri_incident_multiplier: 0.0015,
            max_incident_probability: 0.25,
        }
    }
}

/// Current reputation and food safety state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationState {
    /// Current reputation value [0, 100]
    pub reputation: f32,

    /// Food Safety Risk Index [0, 100]
    /// Higher values mean higher risk of incidents
    pub fsri: f32,

    /// Food quality level [0, 100]
    /// Affects feedback probabilities
    pub food_quality: f32,

    /// Accumulated reputation changes during current day
    /// Reset at day end after applying
    #[serde(skip)]
    pub daily_accumulated: f32,
}

impl Default for ReputationState {
    fn default() -> Self {
        Self {
            reputation: 50.0,
            fsri: 10.0,
            food_quality: 60.0,
            daily_accumulated: 0.0,
        }
    }
}

impl ReputationState {
    /// Apply a single feedback impact with player response
    /// Returns the actual reputation delta applied
    pub fn apply_feedback_impact(
        &mut self,
        base_impact: f32,
        response_score: f32,
        config: &ReputationConfig,
    ) -> f32 {
        // Use different formulas for positive and negative base impacts
        let delta = if base_impact >= 0.0 {
            // Positive feedback: response_score amplifies the benefit
            base_impact * (1.0 + config.response_factor * response_score)
        } else {
            // Negative feedback: positive response_score reduces the harm
            base_impact * (1.0 - config.response_factor * response_score)
        };

        // Clamp to single event limit
        let clamped = delta.clamp(-config.max_single_change, config.max_single_change);

        // Add to daily accumulation
        self.daily_accumulated += clamped;

        clamped
    }

    /// Apply daily accumulated changes and reset for next day
    pub fn apply_daily_update(&mut self, config: &ReputationConfig) {
        // Clamp daily total
        let clamped = self
            .daily_accumulated
            .clamp(-config.max_daily_change, config.max_daily_change);

        // Update reputation
        self.reputation = (self.reputation + clamped).clamp(0.0, 100.0);

        // Reset accumulator
        self.daily_accumulated = 0.0;
    }

    /// Calculate incident probability for current FSRI
    pub fn incident_probability(&self, config: &ReputationConfig) -> f32 {
        (self.fsri * config.fsri_incident_multiplier).min(config.max_incident_probability)
    }
}
