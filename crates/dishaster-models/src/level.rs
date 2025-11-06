use rustc_hash::FxHashSet;

use super::prelude::*;
use crate::{DinerProfile, DinerRandomizerModel, WindowConfiguration};

/// Complete level configuration defining the game scenario
#[derive(Debug, Clone, Deserialize)]
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

    /// Reference to the canteen model
    pub canteen: ModelId,
    /// Player-configured window setups
    pub window_configurations: Vec<WindowConfiguration>,
    /// Placement of dining tables
    pub table_placements: Vec<TablePlacement>,
    /// Placement of tray dispensers
    pub tray_dispenser_placements: Vec<DispenserPlacement>,
    /// Placement of chopstick dispensers
    pub chopstick_dispenser_placements: Vec<DispenserPlacement>,
    /// Placement of dish collectors
    pub collector_placements: Vec<CollectorPlacement>,

    /// Persistent diner pool (accumulated across days, not serialized in config)
    /// This field is populated at runtime from persistence layer
    #[serde(skip)]
    pub persistent_diner_pool: Vec<DinerProfile>,

    /// Set of hint IDs that have been shown (populated at runtime from persistence layer)
    #[serde(skip)]
    pub shown_hints: FxHashSet<String>,
}

impl HasId for LevelConfig {
    fn id(&self) -> &ModelId {
        &self.id
    }
}

/// Physical placement configuration for dining tables
#[derive(Debug, Clone, Deserialize)]
pub struct TablePlacement {
    /// Reference to table model
    pub model: ModelId,
    /// Center position in the canteen
    pub center_pos: Vec2,
}

/// Physical placement configuration for item dispensers
#[derive(Debug, Clone, Deserialize)]
pub struct DispenserPlacement {
    /// Reference to dispenser model
    pub model: ModelId,
    /// Center position in the canteen
    pub center_pos: Vec2,
}

/// Physical placement configuration for dish collectors
#[derive(Debug, Clone, Deserialize)]
pub struct CollectorPlacement {
    /// Reference to collector model
    pub model: ModelId,
    /// Center position in the canteen
    pub center_pos: Vec2,
}
