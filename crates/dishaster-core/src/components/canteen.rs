#![allow(unused)]

use crate::{models::*, prelude::*};

/// Food service window component containing static configuration and player settings
#[derive(Component)]
pub struct Window {
    /// Static configuration reference
    pub service_template: ModelHandle<WindowServiceModel>,
    /// Index of this window within the canteen
    pub slot_index: usize,
    /// Position in the canteen (should correspond to canteen layout)
    pub location: XSegment,
    /// Whether the window is currently closed
    pub disabled: bool,
}

/// Dining table component for customer seating areas
#[derive(Component)]
pub struct DiningTable {
    /// Identifier of the table model
    pub model_id: ModelId,
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
    /// Area in front of the dispenser where diners can receive items
    pub reception_area: Rect,
    /// Current number of items in stock
    pub current_stock: u32,
    /// What type of items this dispenser provides
    pub dispenser_type: DispenserType,
    /// Whether a refill request is pending
    pub refill_pending: bool,
}

/// Dish collection point component for used plates and trays
#[derive(Component)]
pub struct DishCollector {
    /// Reference to collector model configuration
    pub model: ModelHandle<CollectorModel>,
    /// Physical center position of the collector
    pub center_pos: Vec2,
    /// Area in front of the collector where diners can drop off dishes
    pub reception_area: Rect,
    /// Current number of plates waiting to be processed
    pub current_load: u32,
}
