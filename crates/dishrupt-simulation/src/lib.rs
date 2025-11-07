//! Simulation abstractions and interfaces

use dishrupt_core::EntityId;
pub use dishrupt_core::Tick;

/// Abstraction over simulation features.
pub trait SimulationFeature {
    /// The simulation snapshot.
    type Snapshot;
    /// The simulation command.
    type Command;
    /// The simulation query.
    type Query;
    /// The simulation event.
    type Event;
    /// The simulation response.
    type Response;
    /// The persisted simulation state.
    type Profile;
}

/// Interface for simulation implementations
pub trait ISimulation<F: SimulationFeature> {
    /// Get the root entity of the display hierarchy
    fn root_entity(&self) -> EntityId;

    /// Advance the simulation by one time step
    fn tick(&mut self);

    /// Create a snapshot of the current simulation state for serialization or debugging
    fn snapshot(&mut self) -> F::Snapshot;

    /// Retrieve all events that occurred after the last poll
    fn poll_events(&mut self) -> Vec<F::Event>;

    /// Retrieve all query responses that occurred after the last poll
    fn poll_responses(&mut self) -> Vec<F::Response>;

    /// Apply a high-level control command from the client runtime.
    fn command(&mut self, command: F::Command);

    /// Apply a high-level query from the client runtime.
    fn query(&mut self, query: F::Query);

    /// Persist the current simulation state for saving/loading
    fn persist(&mut self) -> F::Profile;
}
