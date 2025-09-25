use serde::{Deserialize, Serialize};

use crate::{components::Movement, prelude::*};

#[allow(missing_docs)]
#[derive(Bundle)]
pub struct DinerBundle {
    pub diner: Diner,
    pub state: DinerState,
    pub targets: DinerTargets,
    pub movement: Movement,
}

/// Core diner identity
#[derive(Component)]
pub struct Diner {
    /// Unique identifier for this diner instance. Useless yet.
    pub id: u32,
}

/// Runtime diner state
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
#[derive(Component, Default)]
pub struct DinerTargets {
    /// Window the diner is currently observing
    pub observing_window: Option<Entity>,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DinerStateType {
    /// Entering the canteen and moving to an observation area.
    Entering,
    /// Wandering to observe different windows before making a choice.
    Observing,
    /// Pausing to decide on a window after observation.
    Deciding,
    /// Moving towards the chosen window.
    MovingToWindow,
    /// Arrived at the window. Currently transitions directly to leaving.
    AtWindow,
    /// Leaving the canteen.
    Leaving,
}
