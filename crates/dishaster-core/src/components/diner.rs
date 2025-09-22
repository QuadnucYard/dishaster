use serde::{Deserialize, Serialize};

use crate::{models::*, prelude::*};

/// Core diner identity - links to static configuration
#[derive(Component)]
pub struct Diner {
    /// Reference to diner's static configuration
    pub archetype: ModelHandle<DinerModel>,
    /// Unique identifier for this diner instance
    pub id: u32,
}

/// Runtime diner state - only mutable data
#[derive(Component)]
pub struct DinerState {
    /// Current state in the state machine
    pub current: DinerStateType,
    /// State entry time for timing
    pub state_timer: f32,
    /// Current satisfaction level
    pub satisfaction: f32,
}

/// Diner's current targets and decisions
#[derive(Component)]
pub struct DinerTargets {
    /// Currently chosen window entity
    pub chosen_window: Option<Entity>,
    /// Currently chosen table entity
    pub chosen_table: Option<Entity>,
}

/// Persistent memory component for diner data across days
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct DinerMemory {
    /// Total number of visits to the canteen
    pub total_visits: u32,
    /// Last day this diner visited
    pub last_visit_day: u32,
    /// Average satisfaction from previous visits
    pub average_satisfaction: f32,
    /// Learned preferences and adaptations
    pub learned_preferences: Vec<EcoString>,
}

/// State machine for diner behavior
#[derive(Debug, Clone)]
pub enum DinerStateType {
    /// Entering the canteen, moving to observation point
    Entering,
    /// Observing windows to make decisions
    Observing,
    /// Making decision about which window to choose
    Deciding,
    /// Moving towards chosen window
    MovingToWindow,
    /// Being served at the window
    BeingServed,
    /// Looking for an available table
    LookingForTable,
    /// Moving towards chosen table
    MovingToTable,
    /// Eating at the table
    EatingAtTable,
    /// Returning plate to return area
    ReturningPlate,
    /// Leaving the canteen
    Leaving,
}
