use super::prelude::*;

/// Type of feedback trigger that caused the complaint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackTrigger {
    /// No dishes appealed to the diner
    NoAppealingDish,
    /// Queue too long / exceeded patience
    QueueTooLong,
    /// Missing utensils (tray or chopsticks)
    MissingUtensils,
    /// Dish appearance below expectation
    AppearanceNotAsExpected,
    /// Contamination detected
    Contamination,
    /// Dish tastes bad
    BadTaste,
    /// Still hungry after meal
    StillHungry,
}

/// Configuration thresholds for triggering feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
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
/// TODO: unused
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialTriggerProbabilities {
    /// Base probability for missing utensils
    pub missing_utensils: f32,
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
            missing_utensils: 0.3,
            appearance_mismatch: 0.2,
            contamination: 0.6,
            bad_taste: 0.3,
            still_hungry: 0.1,
        }
    }
}
