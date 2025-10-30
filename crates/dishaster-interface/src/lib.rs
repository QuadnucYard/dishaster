//! Channel definitions for communicating with the Dishaster simulation.

pub mod command;
pub mod event;
pub mod query;
pub mod response;
pub mod snapshots;

use dishrupt_core::EntityId;

pub use crate::{
    command::SimCommand, event::SimEvent, query::SimQuery, response::SimResponse,
    snapshots::Snapshot,
};

/// Type alias for simulation tick count
pub type Tick = u32;

/// Interface for simulation implementations
pub trait ISimulation {
    /// Get the root entity of the display hierarchy
    fn root_entity(&self) -> EntityId;

    /// Advance the simulation by one time step
    fn tick(&mut self);

    /// Create a snapshot of the current simulation state for serialization or debugging
    fn snapshot(&mut self) -> Snapshot;

    /// Retrieve all events that occurred after the last poll
    fn poll_events(&mut self) -> Vec<SimEvent>;

    /// Retrieve all query responses that occurred after the last poll
    fn poll_responses(&mut self) -> Vec<SimResponse>;

    /// Apply a high-level control command from the client runtime.
    fn command(&mut self, command: SimCommand);

    /// Apply a high-level query from the client runtime.
    fn query(&mut self, query: SimQuery);
}
