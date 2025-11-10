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
