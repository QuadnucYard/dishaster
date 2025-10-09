use crate::{models::*, prelude::*};

/// Food service window component containing static configuration and player settings
#[derive(Component)]
pub struct Window {
    /// Static configuration reference
    pub service_template: ModelHandle<WindowServiceModel>,
    /// Player configuration
    pub config: WindowConfiguration,
    /// Position in the canteen (should correspond to canteen layout)
    pub location: XSegment,
}

/// Active dishes in a window - separate component for better data locality
#[derive(Component)]
pub struct WindowDishes {
    /// Currently available dishes in this window
    pub dishes: Vec<ActiveDish>,
}

/// Dining table component for customer seating areas
#[derive(Component)]
pub struct DiningTable {
    /// Reference to table model configuration
    pub model: ModelHandle<TableModel>,
    /// Physical center position of the table
    pub center_pos: Vec2,
    /// World-space positions diners should move to when seating
    pub seat_positions: Vec<Vec2>,
    /// Which seats are currently occupied (stores occupant entity)
    pub occupants: Vec<Option<Entity>>,
    /// Current dirtiness level of the table
    pub dirtiness: f32,
}

/// Item dispenser component for trays and chopsticks
#[derive(Component)]
pub struct Dispenser {
    /// Reference to dispenser model configuration
    pub model: ModelHandle<DispenserModel>,
    /// Physical center position of the dispenser
    pub center_pos: Vec2,
    /// Current number of items in stock
    pub current_stock: u32,
    /// What type of items this dispenser provides
    pub dispenser_type: DispenserType,
}

/// Types of items that dispensers can provide
#[derive(Debug, Clone, PartialEq)]
pub enum DispenserType {
    /// Dispenses serving trays
    Tray,
    /// Dispenses chopsticks
    Chopstick,
}

/// Dish collection point component for used plates and trays
#[derive(Component)]
pub struct DishCollector {
    /// Reference to collector model configuration
    pub model: ModelHandle<CollectorModel>,
    /// Physical center position of the collector
    pub center_pos: Vec2,
    /// Current number of plates waiting to be processed
    pub current_load: u32,
}
