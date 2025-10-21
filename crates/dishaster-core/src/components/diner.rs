use serde::{Deserialize, Serialize};

use crate::{components::Movement, prelude::*};

#[allow(missing_docs)]
#[derive(Bundle)]
pub struct DinerBundle {
    pub diner: Diner,
    pub state: DinerState,
    pub goal: DinerGoalState,
    pub targets: DinerTargets,
    pub movement: Movement,
}

/// Core diner identity
#[derive(Component)]
pub struct Diner {
    /// Unique identifier for this diner instance. Useless yet.
    pub id: u32,
}

/// Current state of the diner
#[derive(Component, Default)]
pub struct DinerState {
    /// Whether the diner has a tray
    pub has_tray: bool,
    /// Whether the diner has chopsticks
    pub has_chopsticks: bool,
    /// Whether the diner has been served
    pub is_served: bool,
}

/// Current goal state of the diner
#[derive(Component)]
pub struct DinerGoalState {
    /// Current goal
    current: DinerGoal,
    /// Optional next goal to transition to in the next update
    pending: Option<DinerGoal>,
    /// State entry time for timing. Internal use only.
    pub timer: f32,
}

impl Default for DinerGoalState {
    fn default() -> Self {
        Self {
            current: DinerGoal::Enter,
            pending: None,
            timer: 0.0,
        }
    }
}

impl DinerGoalState {
    /// Check if the diner is currently in the given goal state.
    pub fn is(&self, goal: DinerGoal) -> bool {
        self.current == goal
    }

    /// Get the current goal state.
    pub fn current(&self) -> DinerGoal {
        self.current
    }

    /// Transition to the next goal state, resetting the timer.
    pub fn update(&mut self, next_goal: DinerGoal) {
        self.pending = Some(next_goal);
    }

    /// Step the internal timer and apply any pending goal transitions.
    pub fn step(&mut self, delta_time: f32) {
        if let Some(next) = self.pending.take() {
            self.current = next;
            self.timer = 0.0;
        }
        self.timer += delta_time;
    }

    /// Reset the state timer to zero.
    pub fn reset_timer(&mut self) {
        self.timer = 0.0;
    }
}

/// Possible goals for a diner in the canteen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DinerGoal {
    /// Just spawned, moving into the canteen
    Enter,
    /// Observing available windows to decide
    Observe,
    /// Deciding which window to approach
    DecideWindow,
    /// Moving to pick up a tray
    PickTray,
    /// Moving to pick up chopsticks
    PickChopsticks,
    /// Moving to the chosen window
    QueueForWindow,
    /// Being served at the window
    GetServed,
    /// Moving to find a seat
    FindSeat,
    /// Moving to the assigned seat
    MoveToSeat,
    /// Eating the meal at the seat
    Eat,
    /// Moving to the dish return area
    ReturnDishes,
    /// Leaving the canteen
    Leave,
}

/// Diner's current targets and decisions
#[derive(Component, Default)]
pub struct DinerTargets {
    /// Window the diner is currently observing
    pub observing_window: Option<Entity>,
    /// Currently chosen window entity
    pub chosen_window: Option<Entity>,
    /// Currently chosen tray dispenser entity
    pub tray_target: Option<Entity>,
    /// Currently chosen chopstick dispenser entity
    pub chopstick_target: Option<Entity>,
    /// Currently chosen table entity and seat index
    pub chosen_seat: Option<(Entity, usize)>,
    /// Dish collector the diner will visit after eating
    pub collector_target: Option<Entity>,
    /// Exit target position to leave the canteen
    pub exit_target: Option<()>,
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
