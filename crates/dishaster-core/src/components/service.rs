use crate::{models::ModelId, prelude::*};

/// Runtime service session data while the diner is at the counter.
#[derive(Component)]
pub struct ServiceSession {
    /// Window entity currently serving the diner.
    pub window: Entity,
    /// Staff entity assigned to the session, if available.
    pub staff: Option<Entity>,
    /// Current progress in the counter interaction.
    pub stage: ServiceStage,
    /// Requested dish information for feedback and billing.
    pub request: Option<ServiceRequest>,
    /// Timestamp when the session started.
    pub started_at: f64,
}

impl ServiceSession {
    /// Create a new service session for the given window.
    pub fn new(window: Entity, now: f64) -> Self {
        Self {
            window,
            staff: None,
            stage: ServiceStage::AssignStaff,
            request: None,
            started_at: now,
        }
    }
}

/// Requested dish details chosen by the diner.
#[derive(Debug, Clone)]
pub struct ServiceRequest {
    /// Identifier of the dish model.
    pub dish_id: ModelId,
    /// Slot index within the window layout.
    pub dish_slot: usize,
    /// Display name of the ordered dish.
    pub dish_name: EcoString,
    /// Baseline serving time derived from the dish model.
    pub base_service_time: f32,
}

/// Service progress states while the diner stays at the counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStage {
    /// Waiting to secure an available staff member.
    AssignStaff,
    /// Diner has spoken and is waiting for staff response.
    WaitingForStaffResponse,
    /// Staff acknowledged and is preparing the dish.
    WaitingForDish,
    /// Service completed and ready to transition to the next diner state.
    Completed,
}
