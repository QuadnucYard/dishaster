//! Diner randomization configurations

use super::prelude::*;

/// Configuration for pool behavior and decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DinerPoolConfig {
    /// Initial pool size when creating from scratch
    pub initial_pool_size: usize,
    /// Memory decay rate per day (0..1, e.g., 0.95 = 5% decay)
    pub memory_decay_rate: f32,
    /// Tag preference decay rate per day (0..1, e.g., 0.98 = 2% decay)
    pub tag_decay_rate: f32,
    /// Probability modifier for satisfied diners (overall_like >= 0.0)
    pub high_satisfaction_visit_rate: f32,
    /// Probability modifier for dissatisfied diners (overall_like < 0.0)
    pub low_satisfaction_visit_rate: f32,
}

impl Default for DinerPoolConfig {
    fn default() -> Self {
        Self {
            initial_pool_size: 1000,
            memory_decay_rate: 0.95,
            tag_decay_rate: 0.98,
            high_satisfaction_visit_rate: 0.75,
            low_satisfaction_visit_rate: 0.25,
        }
    }
}

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
    /// Range for maximum satiation (stomach capacity)
    pub max_satiation: MinMax<f32>,
    /// Range for eating speed multiplier
    pub eating_speed: MinMax<f32>,
}

/// Ranges for randomizing appearance parts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceRanges {
    /// Number of available head variants
    pub head_variants: u8,
    /// Number of available upper garment variants
    pub upper_garment_variants: u8,
    /// Number of available lower garment variants
    pub lower_garment_variants: u8,
    /// Number of available hand variants
    pub hand_variants: u8,
    /// Number of available shoe variants
    pub shoe_variants: u8,
}

impl Default for AppearanceRanges {
    fn default() -> Self {
        Self {
            head_variants: 4,
            upper_garment_variants: 5,
            lower_garment_variants: 4,
            hand_variants: 3,
            shoe_variants: 3,
        }
    }
}
