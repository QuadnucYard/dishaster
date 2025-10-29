//! Simulation runners for Dishaster simulations.

#[cfg(feature = "threaded")]
mod r#async;
mod sync;

#[cfg(feature = "threaded")]
pub use r#async::AsyncSimulationRunner;
use dishaster_interface::*;
pub use sync::SyncSimulationRunner;

/// Abstraction over simulation runners.
pub trait SimulationRunner {
    /// Advance the simulation by the given delta time.
    /// Returns the snapshot if the simulation was advanced (i.e., accumulator reached the tick threshold).
    ///
    /// # Arguments
    /// * `dt` - Delta time in seconds to advance the simulation
    ///
    /// # Returns
    /// * `Some(Snapshot)` if the simulation ticked and produced a new snapshot
    /// * `None` if the accumulator hasn't reached the tick threshold yet
    fn tick(&mut self, dt: f64) -> Option<SnapshotFrame>;

    /// Force advance the simulation by one tick, regardless of accumulator.
    /// This is useful for testing or when you need immediate advancement.
    ///
    /// # Returns
    /// The new snapshot after the forced tick
    fn force_tick(&mut self) -> Snapshot;

    /// Forward a simulation command to the underlying simulation immediately.
    fn send_command(&mut self, command: SimCommand);

    /// Forward a simulation query to the underlying simulation immediately.
    fn send_query(&mut self, query: SimQuery);

    /// Retrieve the current ticks-per-second target.
    fn tps(&self) -> f64;

    /// Update the target ticks-per-second rate used for fixed-step advancement.
    fn set_tps(&mut self, tps: f64);
}

/// A snapshot frame paired with the number of simulation ticks it represents.
/// This is the unit sent over the channel from the sim thread to the main thread.
pub struct SnapshotFrame {
    /// Number of ticks advanced in this frame.
    pub ticks: Tick,
    /// The simulation snapshot after these ticks.
    pub snapshot: Snapshot,
    /// Events that occurred during these ticks.
    pub events: Vec<SimEvent>,
    /// Responses to queries made during these ticks.
    pub responses: Vec<SimResponse>,
}

impl SnapshotFrame {
    fn extend(&mut self, other: SnapshotFrame) {
        self.ticks = other.ticks;
        self.snapshot = other.snapshot;
        self.events.extend(other.events);
    }
}

fn extend_frame(current: &mut Option<SnapshotFrame>, other: SnapshotFrame) {
    match current {
        Some(snap) => snap.extend(other),
        None => *current = Some(other),
    }
}
