use dishrupt_core::display::DisplayModel;

use super::prelude::*;

/// Physical layout model for the dining hall structure
#[derive(Debug, Clone, Deserialize)]
pub struct CanteenModel {
    /// Unique identifier for this canteen layout
    pub id: ModelId,
    /// Total width of the dining hall in meters
    pub width: Meters,
    /// Total height of the dining hall in meters
    pub height: Meters,

    /// Y coordinate where customers enter/exit the hall
    pub entrances_y: Meters,
    /// X-axis ranges where customers can enter/exit the hall
    pub entrances: Vec<XRange>,

    /// Y coordinate where food service windows are located
    pub windows_y: Meters,
    /// X-axis ranges where food service windows are positioned
    pub windows: Vec<XRange>,

    /// Display model
    pub display: DisplayModel,
}

impl HasId for CanteenModel {
    fn id(&self) -> &ModelId {
        &self.id
    }
}

/// Configuration model for dining tables
#[derive(Debug, Clone, Deserialize)]
pub struct TableModel {
    /// Unique identifier for this table type
    pub id: ModelId,
    /// Physical dimensions of the table
    pub size: Size,
    /// Number of seats at this table. We assume that all seats are on one side for simplicity.
    pub seats: usize,
    /// Comfort rating affecting customer satisfaction
    pub comfort_level: f32,
}

impl HasId for TableModel {
    fn id(&self) -> &ModelId {
        &self.id
    }
}

/// Configuration model for item dispensers (trays, chopsticks)
#[derive(Debug, Clone, Deserialize)]
pub struct DispenserModel {
    /// Unique identifier for this dispenser type
    pub id: ModelId,
    /// Physical dimensions of the dispenser
    pub size: Size,
    /// Maximum capacity
    pub capacity: u32,
    /// Initial stock of items
    pub initial_stock: u32,
    /// Time required to dispense
    pub processing_time: Seconds,
}

impl HasId for DispenserModel {
    fn id(&self) -> &ModelId {
        &self.id
    }
}

/// Configuration model for dish collection stations
#[derive(Debug, Clone, Deserialize)]
pub struct CollectorModel {
    /// Unique identifier for this collector type
    pub id: ModelId,
    /// Physical dimensions of the collector
    pub size: Size,
    /// Maximum number of dishes that can be stored
    pub capacity: u32,
    /// Processing capacity per time unit
    pub processing_capacity: Seconds,
}

impl HasId for CollectorModel {
    fn id(&self) -> &ModelId {
        &self.id
    }
}
