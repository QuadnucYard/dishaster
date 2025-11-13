use super::prelude::*;

/// Different feedback topics that can be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum FeedbackTopic {
    /// No dishes appealed to the diner
    Appeal,
    /// Queue too long / exceeded patience
    Queue,
    /// Missing tableware (tray or chopsticks)
    Tableware,
    /// Dish below expectation
    Quality,
    /// Food hygiene issues encountered
    Hygiene,
    /// Dish tastes bad
    Taste,
    /// Still hungry after meal
    Hunger,
    /// Positive feedback
    Praise,
}

/// Configuration thresholds for triggering feedback
#[derive(Debug, Clone, Deserialize)]
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
        }
    }
}

/// Trial trigger probabilities for different feedback types
///
/// These base probabilities are modified by diner personality
/// (especially confrontational trait) to determine if a trial is triggered.
/// TODO: this is unused
#[derive(Debug, Clone, Deserialize)]
pub struct TrialTriggerProbabilities {
    /// Base probability for missing tableware
    pub missing_tableware: f32,
    /// Base probability for appearance not as expected
    pub appearance_mismatch: f32,
    /// Base probability for contamination
    pub contamination: f32,
    /// Base probability for bad taste (only for confrontational > 0.6)
    pub bad_taste: f32,
    /// Base probability for still hungry
    pub still_hungry: f32,
}

impl Default for TrialTriggerProbabilities {
    fn default() -> Self {
        Self {
            missing_tableware: 0.3,
            appearance_mismatch: 0.2,
            contamination: 0.6,
            bad_taste: 0.3,
            still_hungry: 0.1,
        }
    }
}

/// Configuration for reputation system
#[derive(Debug, Clone, Deserialize)]
pub struct ReputationConfig {
    /// Base impact values for each feedback topic on reputation
    /// Positive values increase reputation, negative values decrease it
    pub base_impacts: FeedbackBaseImpacts,

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
        Self {
            base_impacts: FeedbackBaseImpacts::default(),
            response_factor: 0.6,
            max_single_change: 8.0,
            max_daily_change: 12.0,
            fsri_incident_multiplier: 0.0015,
            max_incident_probability: 0.25,
        }
    }
}

/// Base reputation impact values for each feedback topic
#[derive(Debug, Clone, Deserialize)]
pub struct FeedbackBaseImpacts {
    /// Positive feedback base impact
    pub praise: f32,
    /// No appealing dish base impact
    pub appeal: f32,
    /// Queue too long base impact
    pub queue: f32,
    /// Missing tableware base impact
    pub tableware: f32,
    /// Dish quality issue base impact
    pub quality: f32,
    /// Food hygiene issue base impact (most severe)
    pub hygiene: f32,
    /// Bad taste base impact
    pub taste: f32,
    /// Still hungry base impact
    pub hunger: f32,
}

impl Default for FeedbackBaseImpacts {
    fn default() -> Self {
        Self {
            praise: 3.0,
            appeal: -2.0,
            queue: -3.0,
            tableware: -2.5,
            quality: -4.0,
            hygiene: -12.0, // Most severe
            taste: -3.5,
            hunger: -2.0,
        }
    }
}

impl FeedbackBaseImpacts {
    /// Get base impact for a specific feedback topic
    pub fn get(&self, topic: FeedbackTopic) -> f32 {
        match topic {
            FeedbackTopic::Praise => self.praise,
            FeedbackTopic::Appeal => self.appeal,
            FeedbackTopic::Queue => self.queue,
            FeedbackTopic::Tableware => self.tableware,
            FeedbackTopic::Quality => self.quality,
            FeedbackTopic::Hygiene => self.hygiene,
            FeedbackTopic::Taste => self.taste,
            FeedbackTopic::Hunger => self.hunger,
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
