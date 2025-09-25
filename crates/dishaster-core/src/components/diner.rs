use dishaster_models::DinerModel;
use serde::{Deserialize, Serialize};

use crate::{
    components::{ComponentWrapper, Movement},
    prelude::*,
};

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

/// Component wrapper for DinerModel
pub type DinerModelComp = ComponentWrapper<DinerModel>;

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

/// Marker component while a diner participates in a queue at a window
#[derive(Component)]
pub struct QueueParticipant {
    /// Service window this diner intends to order from
    pub window: Entity,
    /// Simulation time when the diner joined the queue (seconds)
    pub joined_at: f64,
    /// Current zero-based index within the queue ordering
    pub slot_index: usize,
}

impl QueueParticipant {
    /// Create a new queue participant entry at the moment a diner joins the queue
    pub fn new(window: Entity, joined_at: f64) -> Self {
        Self {
            window,
            joined_at,
            slot_index: 0,
        }
    }
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
    /// Standing in the queue and waiting to reach the counter.
    Queueing,
    /// Currently being served at the counter.
    BeingServed,
    /// Leaving the canteen.
    Leaving,
}
