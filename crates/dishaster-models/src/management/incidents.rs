use std::ops::Range;

use crate::prelude::*;

/// Template for mislabeling prices incident
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "MislabelPrice")]
pub struct MislabelPriceTemplate {
    /// Range of number of overpriced items
    pub num_range: Range<usize>,

    /// Range of overprice rate
    pub overprice_rate_range: Range<f32>,
}

/// Model for mislabeling prices incident
#[derive(Debug, Clone)]
pub struct MislabelPriceModel {
    /// Overpriced rates
    pub overpriced_rates: Vec<f32>,
}

/// Template for attraction change incident (can increase or decrease diner visit willingness)
/// Used for both positive events (increase) and negative events (decrease)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "AttractionChange")]
pub struct AttractionChangeTemplate {
    /// Attraction multiplier (e.g., 1.3 = 30% more, 0.7 = 30% less likely to visit)
    pub attraction_multiplier_range: Range<f32>,
}

/// Model for attraction change incident
#[derive(Debug, Clone)]
pub struct AttractionChangeModel {
    /// Attraction multiplier
    pub attraction_multiplier: f32,
}

/// Template for temporary crowd incident (burst of temporary diners)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "TemporaryCrowd")]
pub struct TemporaryCrowdTemplate {
    /// Number of temporary diners
    pub num_diners_range: Range<usize>,
    /// Peak arrival time (in hours from day start)
    pub peak_time_range: Range<f32>,
    /// Standard deviation for arrival time distribution (in hours)
    pub time_stddev: f32,
}

/// Model for temporary crowd incident
#[derive(Debug, Clone)]
pub struct TemporaryCrowdModel {
    /// Number of temporary diners
    pub num_diners: usize,
    /// Peak arrival time (in hours from day start)
    pub peak_time: f32,
    /// Standard deviation for arrival time distribution
    pub time_stddev: f32,
}
