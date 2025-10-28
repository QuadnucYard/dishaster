use dishrupt_core::{asset::PrefabReference, display::DisplayModel};

use super::prelude::*;

/// Complete diner configuration model used both as component and model definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DinerModel {
    /// Core attributes
    pub attributes: DinerAttributes,
    /// Behavioral parameters
    pub behavior: DinerBehavior,
    /// Extensible properties for future features
    pub properties: DinerProperties,
    /// Display model
    pub display: DisplayModel,
}

/// Core psychological and economic attributes of a diner
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DinerAttributes {
    /// Basic physiological state
    pub hunger: f32,
    /// How long the diner will wait before leaving
    pub patience: f32,
    /// Economic characteristics
    pub economic_capacity: f32,
    /// How much price affects purchase decisions
    pub price_sensitivity: f32,
}

/// Behavioral patterns and timing parameters for diner decision-making
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DinerBehavior {
    /// Decision making
    pub decisiveness: f32,
    /// Ability to adapt to changing conditions
    pub adaptiveness: f32,
    /// Base probability of leaving without purchasing
    pub leave_probability: f32,
    /// Timing parameters
    pub observation_time: Seconds,
    /// Time spent deciding what to order
    pub decision_time: Seconds,
    /// Time spent eating the meal
    pub eating_time: Seconds,
}

/// Extensible properties for future diner features and preferences
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DinerProperties {
    /// Base satisfaction level
    pub base_satisfaction: f32,
    /// Extensible preference system - start simple
    pub preferences: Vec<EcoString>,
}

/// Configuration model for diner randomization parameters
#[derive(Debug, Clone, Deserialize)]
pub struct DinerProviderModel {
    /// Attribute ranges
    pub attributes: DinerAttributeRanges,
    /// Behavior ranges
    pub behavior: DinerBehaviorRanges,
    /// Movement parameters
    pub movement: MovementRanges,
    /// List of display resources
    pub display_res: Vec<PrefabReference>,
}

/// Range definitions for randomizing diner attributes
#[derive(Debug, Clone, Deserialize)]
pub struct DinerAttributeRanges {
    /// Range for hunger levels
    pub hunger: MinMax,
    /// Range for patience levels
    pub patience: MinMax,
    /// Range for economic capacity
    pub economic_capacity: MinMax,
    /// Range for price sensitivity factors
    pub price_sensitivity: MinMax<f32>,
}

/// Range definitions for randomizing diner behavioral parameters
#[derive(Debug, Clone, Deserialize)]
pub struct DinerBehaviorRanges {
    /// Range for decisiveness levels
    pub decisiveness: MinMax<f32>,
    /// Range for adaptiveness levels
    pub adaptiveness: MinMax<f32>,
    /// Range for base leave probabilities
    pub leave_probability: MinMax<f32>,
    /// Range for observation time durations
    pub observation_time: MinMax<Seconds>,
    /// Range for decision-making time durations
    pub decision_time: MinMax<Seconds>,
    /// Range for eating time durations
    pub eating_time: MinMax<Seconds>,
}

/// Range definitions for diner movement and physics parameters
#[derive(Debug, Clone, Deserialize)]
pub struct MovementRanges {
    /// Range for normal movement speeds
    pub movement_speed: MinMax<f32>,
    /// Range for avoidance/evasion speeds
    pub avoidance_speed: MinMax<f32>,
    /// Range for arrival detection thresholds
    pub arrival_threshold: MinMax<f32>,
}

// ===================== Enhanced Decision System =====================

/// Fixed personality traits that shape diner behavior
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Personality {
    /// Price sensitivity (0..1, higher = more price sensitive)
    pub frugality: f32,
    /// Willingness to try new dishes (0..1, higher = more adventurous)
    pub adventurous: f32,
    /// Likelihood to question or complain about issues (0..1)
    pub confrontational: f32,
    /// Base patience in seconds
    pub patience_base: f32,
}

/// Current psychological state affecting decision-making
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PsychState {
    /// Current hunger level (0..1, higher = more hungry)
    pub hunger: f32,
    /// Current mood (-1..1, negative = bad mood, positive = good mood)
    pub mood: f32,
    /// Current patience threshold in seconds (dynamically adjusted)
    pub patience_now: f32,
    /// Trust in the canteen (0..1, affects tolerance to issues)
    pub trust: f32,
}

/// Memory of a specific dish from previous experiences
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DishMemory {
    /// Number of times this dish was eaten
    pub times_eaten: u32,
    /// Average rating/satisfaction from eating this dish (-1..1)
    pub avg_rating: f32,
    /// Last time this dish was eaten (simulation time)
    pub last_eaten: Option<f32>,
}

/// Long-term memory persisted across multiple visits
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LongTermMemory {
    /// Preference weights for dish tags (-1..1 per tag)
    /// Positive values indicate preference, negative indicate dislike
    pub like_tags: std::collections::HashMap<EcoString, f32>,
    /// Experience with specific dishes
    pub dish_experience: std::collections::HashMap<ModelId, DishMemory>,
    /// Overall satisfaction with this canteen (0..1)
    pub overall_like: f32,
}

/// Short-term memory for current meal session
#[derive(Debug, Clone, Default)]
pub struct ShortTermMemory {
    /// Windows that have been observed this session
    pub seen_windows: std::collections::HashSet<ModelId>,
    /// Dishes tried in current meal
    pub tried_dishes: Vec<ModelId>,
    /// Perceived price references updated by seeing prices
    pub expected_prices: std::collections::HashMap<ModelId, f32>,
}
