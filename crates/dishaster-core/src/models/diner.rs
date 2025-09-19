use super::prelude::*;

/// Complete diner configuration model used both as component and model definition
#[derive(Component, Debug, Clone, Deserialize, Serialize)]
pub struct DinerModel {
    /// Core attributes
    pub attributes: DinerAttributes,
    /// Behavioral parameters
    pub behavior: DinerBehavior,
    /// Extensible properties for future features
    pub properties: DinerProperties,
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
