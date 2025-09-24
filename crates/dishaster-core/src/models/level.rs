use super::prelude::*;
use crate::models::*;

/// Complete level configuration defining the game scenario
#[derive(Resource, Debug, Clone, Deserialize)]
pub struct LevelConfig {
    /// Unique identifier for this level
    pub id: ModelId,
    /// Which day/level this represents
    #[serde(default)]
    pub day: u32,

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

    /// Diner generation parameters
    pub diner_provider: DinerProviderModel,
    /// Diner spawning timing configuration
    pub diner_spawner: DinerSpawnerModel,
    /// Random seed for reproducible gameplay
    pub seed: u64,
}

impl HasId for LevelConfig {
    fn id(&self) -> &ModelId {
        &self.id
    }
}

/// Legacy support - can be removed once systems are updated
#[derive(Debug, Clone, Deserialize)]
pub struct ActiveWindowModel {
    /// Reference to service template
    pub service: ModelId,
    /// Position of the window
    pub pos: XRange,
    /// Whether the window is currently open for service
    pub is_open: bool,
    /// Available dishes at this window
    pub dishes: Vec<Option<ServedDish>>,
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

/// Configuration model for diner spawning parameters
#[derive(Debug, Clone, Deserialize)]
pub struct DinerSpawnerModel {
    /// Total duration of the simulation run
    pub run_length: Seconds,
    /// Time range between diner spawns
    pub spawn_interval: MinMax<Seconds>,
}
