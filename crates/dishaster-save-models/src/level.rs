use crate::{DinerProfile, Seed, prelude::*};

/// Day index wrapper
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Day(pub u32);

/// Complete save state for a specific level run
#[derive(Debug, Serialize, Deserialize)]
pub struct LevelSetupState {
    /// Identifier for the level being played
    pub level_id: ModelId,

    /// Current day index
    pub day: Day,

    /// Seed for the day's RNG
    pub seed: Seed,

    /// Player-customized canteen layout and configurations
    pub canteen: CanteenLayoutState,

    /// Persistent diner pool (accumulated across days)
    /// This field is populated at runtime from persistence layer
    pub diner_pool: Vec<DinerProfile>,
}

/// Player-customized canteen layout and configurations
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CanteenLayoutState {
    /// Player-configured window setups
    pub window_configurations: Vec<WindowConfiguration>,

    /// Physical placements of canteen objects
    pub placement: CanteenPlacements,
}

/// Physical placements of canteen objects
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CanteenPlacements {
    /// Placement of dining tables
    #[serde(rename = "table_placements")]
    pub tables: Vec<Placement>,
    /// Placement of tray dispensers
    #[serde(rename = "tray_dispenser_placements")]
    pub tray_dispensers: Vec<Placement>,
    /// Placement of chopstick dispensers
    #[serde(rename = "chopstick_dispenser_placements")]
    pub chopstick_dispensers: Vec<Placement>,
    /// Placement of dish collectors
    #[serde(rename = "collector_placements")]
    pub collectors: Vec<Placement>,
}

/// Physical placement configuration for dish collectors
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Placement {
    /// Center position in the canteen
    pub center_pos: Vec2,
    /// Reference to collector model
    pub model: ModelId,
}

// ===================== Operational Configuration =====================

/// Player's configuration for a specific window instance
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowConfiguration {
    /// Which slot to use
    pub slot_index: usize,
    /// Which service template this uses
    pub service_template: ModelId,
    /// Whether enabled
    pub is_enabled: bool,
    /// Player-selected dishes
    pub dish_assignments: Vec<DishAssignment>,
}

/// Player's assignment of a dish to a specific slot in a window
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DishAssignment {
    /// Which slot to use
    pub slot_index: usize,
    /// Which dish to serve
    pub dish_id: ModelId,
    /// Player-set pricing
    pub pricing: PricingMethod,
}

/// Different pricing strategies for dishes
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum PricingMethod {
    /// Fixed price per serving
    PerPortion(f32),
    /// Price calculated by weight (per kg)
    ByWeight(f32),
}
