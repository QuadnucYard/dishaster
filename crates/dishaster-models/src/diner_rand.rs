//! Diner randomization configurations

use super::{AppearanceRanges, prelude::*};

/// Configuration for randomizing diner traits during pool initialization
#[derive(Debug, Clone, Deserialize)]
pub struct DinerRandomizerModel {
    /// Personality trait ranges
    pub personality: PersonalityRanges,
    /// Dining profile ranges
    pub dining: DiningRanges,
    /// Appearance cosmetic ranges
    #[serde(default)]
    pub appearance: AppearanceRanges,
}

/// Range definitions for randomizing personality traits
#[derive(Debug, Clone, Deserialize)]
pub struct PersonalityRanges {
    /// Range for price sensitivity (frugality)
    pub frugality: MinMax<f32>,
    /// Range for adventurousness
    pub adventurous: MinMax<f32>,
    /// Range for confrontational tendency
    pub confrontational: MinMax<f32>,
    /// Range for base patience in seconds
    pub patience_base: MinMax<f32>,
    /// Range for decisiveness
    pub decisiveness: MinMax<f32>,
    /// Range for adaptiveness
    pub adaptiveness: MinMax<f32>,
}

/// Range definitions for randomizing dining profile
#[derive(Debug, Clone, Deserialize)]
pub struct DiningRanges {
    /// Range for economic capacity
    pub economic_capacity: MinMax<f32>,
    /// Range for eating speed multiplier
    pub eating_speed: MinMax<f32>,
}
