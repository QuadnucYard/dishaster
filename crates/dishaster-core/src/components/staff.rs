use crate::{components::Movement, prelude::*};

/// Component identifying a serving staff member assigned to a window.
#[derive(Component)]
pub struct ServingStaff {
    /// Window entity this staff member serves.
    pub window: Entity,
}

/// Runtime state for a serving staff member handling diners.
#[derive(Component)]
pub struct ServingStaffState {
    /// Current activity status of the staff member.
    pub status: ServingStaffStatus,
    /// Diner entity currently being served if any.
    pub current_session: Option<Entity>,
    /// Simulation timestamp of the latest interaction.
    pub last_update_time: f64,
}

impl Default for ServingStaffState {
    fn default() -> Self {
        Self {
            status: ServingStaffStatus::Idle,
            current_session: None,
            last_update_time: 0.0,
        }
    }
}

impl ServingStaffState {
    /// Check whether the staff member is currently free to take a new diner.
    pub fn is_idle(&self) -> bool {
        matches!(self.status, ServingStaffStatus::Idle)
    }

    /// Reset the staff member to an idle state.
    pub fn reset(&mut self, timestamp: f64) {
        self.status = ServingStaffStatus::Idle;
        self.current_session = None;
        self.last_update_time = timestamp;
    }
}

/// Activity status for serving staff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServingStaffStatus {
    /// Waiting for the next diner.
    Idle,
    /// Actively handling an ordering diner.
    HandlingOrder,
}

/// Convenience bundle for spawning serving staff entities.
#[derive(Bundle)]
pub struct ServingStaffBundle {
    /// Identity component referencing the window served.
    pub staff: ServingStaff,
    /// Mutable runtime state for staff logic.
    pub state: ServingStaffState,
    /// Navigation state used to walk toward diners and service points.
    pub movement: Movement,
}

/// Component linking a queue lane to its assigned staff member.
#[derive(Component)]
pub struct StaffForLane {
    /// The staff entity that serves this lane
    pub staff: Entity,
}

/// Convenience bundle for spawning refill staff entities.
#[derive(Bundle)]
pub struct RefillStaffBundle {
    /// Core staff identity
    pub staff: RefillStaff,
    /// Runtime state
    pub state: RefillStaffState,
    /// Movement component
    pub movement: Movement,
}

/// Component identifying a refill staff member responsible for restocking dispensers.
#[derive(Component)]
pub struct RefillStaff {
    /// Target dispenser entity to refill
    pub target_dispenser: Entity,
    /// Original spawn position to return to after refilling
    pub spawn_pos: Vec2,
}

/// Runtime state for a refill staff member.
#[derive(Component)]
pub struct RefillStaffState {
    /// Current status of the refill staff
    pub status: RefillStaffStatus,
    /// Time spent in current activity
    pub activity_timer: f32,
}

/// Activity status for refill staff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefillStaffStatus {
    /// Moving to the target dispenser
    MovingToDispenser,
    /// Refilling the dispenser
    Refilling,
    /// Returning to spawn point
    Returning,
}
