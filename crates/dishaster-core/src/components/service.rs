#![allow(unused)]

use crate::{models::ModelId, prelude::*};

/// Runtime service session data while the diner is at the counter.
/// This is mounted on the diner entity.
#[derive(Component)]
pub struct ServiceSession {
    /// Window entity currently serving the diner.
    pub window: Entity,
    /// Lane entity the diner is queued at.
    pub lane: Entity,
    /// Staff entity assigned to the session, if available.
    pub staff: Option<Entity>,
    /// Current progress in the counter interaction.
    pub stage: ServiceStage,
    /// Current dish being processed in the order
    pub request: Option<ServiceRequest>,
    /// Timestamp when the session started.
    pub started_at: f64,
    /// Planned order of dishes to get (decided at session start)
    pub planned_order: Vec<ServiceRequest>,
    /// Index of current dish in planned_order
    pub current_dish_index: usize,
}

impl ServiceSession {
    /// Create a new service session for the given window.
    pub fn new(window: Entity, lane: Entity, now: f64) -> Self {
        Self {
            window,
            lane,
            staff: None,
            stage: ServiceStage::AssignStaff,
            request: None,
            started_at: now,
            planned_order: Vec::new(),
            current_dish_index: 0,
        }
    }
}

/// Requested dish details chosen by the diner.
#[derive(Debug, Clone)]
pub struct ServiceRequest {
    /// Identifier of the dish model.
    pub dish_id: ModelId,
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
    /// Dish received, deciding whether to order more
    DecideNextDish,
    /// Payment/checkout stage (刷卡结账)
    Payment,
    /// Service completed and ready to transition to the next diner state.
    Completed,
}
