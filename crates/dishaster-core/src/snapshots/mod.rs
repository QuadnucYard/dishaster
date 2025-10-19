//! Simulation state snapshots for rendering and debugging.

mod dbg;
mod events;

pub use dbg::*;
use dishrupt_core::display::DisplaySnapshot;
pub use events::*;

use crate::Tick;

/// Simulation state snapshot for rendering
pub struct Snapshot {
    /// Simulation timestamp in seconds.
    pub sim_time_seconds: f64,
    /// Total simulation ticks since start.
    pub sim_tick: Tick,
    /// Display graph data for rendering the current frame.
    pub display: Vec<DisplaySnapshot>,
    /// Debug visualization snapshots.
    pub debug: DebugSnapshots,
}
