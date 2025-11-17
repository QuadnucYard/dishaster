use dishaster_save_models::{Day, Placement, Seed, WindowConfiguration};

use super::prelude::*;
use crate::{DinerPoolConfig, DinerRandomizerModel};

/// Complete level configuration defining the game scenario
#[derive(Debug, Deserialize)]
pub struct LevelConfig {
    /// Unique identifier for this level
    pub id: ModelId,
    /// Starting day index
    #[serde(default)]
    pub start_day: Day,
    /// Total duration of the simulation run
    pub run_length: Seconds,
    /// Random seed for reproducible gameplay
    pub seed: Seed,
    /// Entry time when player enters preparation phase (default 11:00:00 = 39600)
    #[serde(default = "default_entry_time")]
    pub entry_time: Seconds,
    /// Start time when service begins (default 11:30:00 = 41400)
    #[serde(default = "default_start_time")]
    pub start_time: Seconds,
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

/// Default entry time: 11:00:00 (39600 seconds since midnight)
fn default_entry_time() -> Seconds {
    11.0 * 3600.0
}

/// Default start time: 11:30:00 (41400 seconds since midnight)
fn default_start_time() -> Seconds {
    11.5 * 3600.0
}
