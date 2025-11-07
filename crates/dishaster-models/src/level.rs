use dishaster_save_models::{Placement, WindowConfiguration};

use super::prelude::*;
use crate::{DinerPoolConfig, DinerRandomizerModel};

/// Complete level configuration defining the game scenario
#[derive(Debug, Deserialize)]
pub struct LevelConfig {
    /// Unique identifier for this level
    pub id: ModelId,
    /// Which day/level this represents
    #[serde(default)]
    pub day: u32,
    /// Total duration of the simulation run
    pub run_length: Seconds,
    /// Random seed for reproducible gameplay
    pub seed: u64,
    /// Diner generation parameters
    pub diner_randomizer: DinerRandomizerModel,

    /// Configuration for the persistent diner pool
    #[serde(default)]
    pub diner_pool: DinerPoolConfig,

    /// Reference to the canteen model
    pub canteen: ModelId,

    // To make the config file flat, we inline these fields from CanteenLayoutState
    /// Player-configured window setups
    pub window_configurations: Vec<WindowConfiguration>,
    /// Placement of dining tables
    pub table_placements: Vec<Placement>,
    /// Placement of tray dispensers
    pub tray_dispenser_placements: Vec<Placement>,
    /// Placement of chopstick dispensers
    pub chopstick_dispenser_placements: Vec<Placement>,
    /// Placement of dish collectors
    pub collector_placements: Vec<Placement>,
}

impl HasId for LevelConfig {
    fn id(&self) -> &ModelId {
        &self.id
    }
}
