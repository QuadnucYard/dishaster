use dishaster_models::*;

use crate::prelude::*;

/// Association of a dish with the service window it is served at
#[derive(Component)]
#[relationship(relationship_target = WindowDishes)]
pub struct ServedAtWindow(pub Entity);

/// Component marking the collection of dishes available at a service window
#[derive(Component)]
#[relationship_target(relationship = ServedAtWindow, linked_spawn)]
pub struct WindowDishes(Vec<Entity>);

/// An active dish available at a service window
#[derive(Component)]
pub struct Dish {
    /// Configuration this came from
    pub assignment: DishAssignment,
    /// Current runtime state
    pub state: DishRuntimeState,
}

/// Dynamic state tracking for dishes during simulation
pub struct DishRuntimeState {
    /// Current quantity available
    pub current_quantity: f32,
    /// Current quality (may degrade)
    pub current_quality: f32,
    /// Current contamination level
    pub contamination_level: f32,
    // TODO: remaining fields
    // /// When last restocked
    // pub last_restocked: f32,
    // /// Times served today
    // pub service_count: u32,
}
