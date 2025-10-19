//! Channel definitions for communicating with the Dishaster simulation.

pub mod commands;
pub mod events;
pub mod snapshots;

use dishaster_models::LevelConfig;
use dishrupt_core::EntityId;

use crate::{commands::SimCommand, events::PresentationEvent, snapshots::Snapshot};

/// Type alias for simulation tick count
pub type Tick = u32;

/// Interface for simulation implementations
pub trait ISimulation {
    /// Get the root entity of the display hierarchy
    fn root_entity(&self) -> EntityId;

    /// Initialize and start a simulation level with the given configuration
    fn start(&mut self, level: LevelConfig);

    /// Advance the simulation by one time step
    fn tick(&mut self);

    /// Create a snapshot of the current simulation state for serialization or debugging
    fn snapshot(&mut self) -> Snapshot;

    /// Retrieve all events that occurred after the last poll
    fn poll_events(&mut self) -> Vec<PresentationEvent>;

    /// Apply a high-level control command from the client runtime.
    fn handle_command(&mut self, command: SimCommand);
}
