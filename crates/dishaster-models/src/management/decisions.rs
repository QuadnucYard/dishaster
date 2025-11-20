use std::ops::RangeInclusive;

use crate::{DispenserType, prelude::*};

/// Template for adding tables decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "AddTables")]
pub struct AddTablesTemplate {
    /// Range of number of tables to add
    pub num_range: RangeInclusive<usize>,
}

/// Model for adding tables decision
#[derive(Debug, Clone)]
pub struct AddTablesModel {
    /// Number of tables to add
    pub num_tables: usize,
}

/// Template for removing tables decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "RemoveTables")]
pub struct RemoveTablesTemplate {
    /// Range of number of tables to add
    pub num_range: RangeInclusive<usize>,
}

/// Model for removing tables decision
#[derive(Debug, Clone)]
pub struct RemoveTablesModel {
    /// Number of tables to add
    pub num_tables: usize,
}

/// Template for disarranging tables decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "DisarrangeTables")]
pub struct DisarrangeTablesTemplate {
    /// Range of number of tables to add
    pub num_range: RangeInclusive<usize>,
}

/// Model for disarranging tables decision
#[derive(Debug, Clone)]
pub struct DisarrangeTablesModel {
    /// Number of tables to add
    pub num_tables: usize,
}

/// Template for adding dispenser decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "AddDispenser")]
pub struct AddDispenserTemplate {
    /// Type of dispenser to add
    pub dispenser_type: DispenserType,
    /// Model ID of the dispenser to add
    pub dispenser_model: ModelId,
}

/// Model for adding dispenser decision
#[derive(Debug, Clone)]
pub struct AddDispenserModel {
    /// Type of dispenser to add
    pub dispenser_type: DispenserType,
    /// Model ID of the dispenser to add
    pub dispenser_model: ModelId,
}

/// Template for opening a window decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "OpenWindow")]
pub struct OpenWindowTemplate {}

/// Model for opening a window decision
#[derive(Debug, Clone)]
pub struct OpenWindowModel {}

/// Template for closing a window decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "CloseWindow")]
pub struct CloseWindowTemplate {}

/// Model for closing a window decision
#[derive(Debug, Clone)]
pub struct CloseWindowModel {}

/// Template for changing window service decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ChangeWindowService")]
pub struct ChangeWindowServiceTemplate {}

/// Model for changing window service decision
#[derive(Debug, Clone)]
pub struct ChangeWindowServiceModel {}

/// Template for playing music decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "PlayMusic")]
pub struct PlayMusicTemplate {
    /// Range of eating time multiplier (e.g., 0.8..1.2 means -20% to +20%)
    pub eating_time_multiplier_range: RangeInclusive<f32>,
    /// Range of satisfaction change
    pub satisfaction_change_range: RangeInclusive<f32>,
}

/// Model for playing music decision
#[derive(Debug, Clone)]
pub struct PlayMusicModel {
    /// Multiplier for eating time (e.g., 0.9 means 10% faster)
    pub eating_time_multiplier: f32,
    /// Change to satisfaction (can be positive or negative)
    pub satisfaction_change: f32,
}

/// Campaign target type for decisions
#[derive(Debug, Clone, Deserialize)]
pub enum DecisionCampaignTarget {
    /// Advertise for the entire canteen
    Canteen,
    /// Advertise for a specific window service (will be randomly selected)
    Window,
}

/// Template for advertising campaign decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "AdvertiseCampaign")]
pub struct AdvertiseCampaignTemplate {
    /// Target of the campaign
    pub target: DecisionCampaignTarget,
    /// Range of attraction boost multiplier (e.g., 1.2..1.5 means +20% to +50%)
    pub attraction_boost_range: RangeInclusive<f32>,
    /// Number of days the effect lasts
    pub duration_days: u32,
    /// Decay rate per day (0..1, e.g., 0.2 means loses 20% effectiveness per day)
    pub decay_rate: f32,
}

/// Model for advertising campaign decision
#[derive(Debug, Clone)]
pub struct AdvertiseCampaignModel {
    /// Target of the campaign
    pub target: DecisionCampaignTarget,
    /// Initial attraction boost multiplier
    pub attraction_boost: f32,
    /// Number of days remaining
    pub days_remaining: u32,
    /// Decay rate per day
    pub decay_rate: f32,
    /// Target window service ID (only used when target is Window)
    pub target_window: Option<ModelId>,
}

/// Template for motivational slogan decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "AddMotivationalSlogan")]
pub struct AddMotivationalSloganTemplate {
    /// Range of trust threshold (diners with trust below this will be unhappy)
    pub trust_threshold_range: RangeInclusive<f32>,
    /// Satisfaction boost for trusting diners
    pub satisfaction_boost_range: RangeInclusive<f32>,
    /// Satisfaction penalty for distrusting diners
    pub satisfaction_penalty_range: RangeInclusive<f32>,
}

/// Model for motivational slogan decision
#[derive(Debug, Clone)]
pub struct AddMotivationalSloganModel {
    /// Trust threshold (diners with trust below this will be unhappy)
    pub trust_threshold: f32,
    /// Satisfaction boost for trusting diners
    pub satisfaction_boost: f32,
    /// Satisfaction penalty for distrusting diners
    pub satisfaction_penalty: f32,
}

/// Template for supplying crab dish decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "SupplyCrab")]
pub struct SupplyCrabTemplate {
    /// Probability for diners to trigger crab topic trial (0.0..1.0)
    pub trial_probability: f32,
}

/// Model for supplying crab dish decision
#[derive(Debug, Clone)]
pub struct SupplyCrabModel {
    /// Probability for diners to trigger crab topic trial
    pub trial_probability: f32,
}

/// Template for improving dish quality decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ImproveDishQuality")]
pub struct ImproveDishQualityTemplate {
    /// Range of quality multiplier (e.g., 1.1..1.3 means +10% to +30% quality)
    pub quality_multiplier_range: RangeInclusive<f32>,
}

/// Model for improving dish quality decision
#[derive(Debug, Clone)]
pub struct ImproveDishQualityModel {
    /// Multiplier for dish quality (e.g., 1.2 means 20% better quality)
    pub quality_multiplier: f32,
}

/// Template for reducing serving time decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ReduceServingTime")]
pub struct ReduceServingTimeTemplate {
    /// Range of serving time multiplier (e.g., 0.7..0.9 means -30% to -10% time)
    pub serving_time_multiplier_range: RangeInclusive<f32>,
}

/// Model for reducing serving time decision
#[derive(Debug, Clone)]
pub struct ReduceServingTimeModel {
    /// Multiplier for serving time (e.g., 0.8 means 20% faster)
    pub serving_time_multiplier: f32,
}
