//! Simulation state snapshots for rendering and debugging.

mod dbg;
mod events;

pub use dbg::*;
use dishrupt_core::display::DisplaySnapshot;
pub use events::*;

use crate::Tick;

/// Simulation state snapshot for rendering
pub struct Snapshot {
    /// Display graph data for rendering the current frame.
    pub display: Vec<DisplaySnapshot>,
    /// Per-agent movement debug data collected for visualization.
    pub movement_debug: Option<Vec<MovementDebugSnapshot>>,
    /// Queue lane debug data collected for visualization.
    pub queue_debug: Option<Vec<QueueLaneDebugSnapshot>>,
    /// Collision grid occupancy data when debug is enabled.
    pub collision_debug: Option<CollisionGridDebugSnapshot>,
    /// Crowd cost field data when debug is enabled.
    pub crowd_debug: Option<CrowdFieldDebugSnapshot>,
    /// Simulation timestamp in seconds.
    pub sim_time_seconds: f64,
    /// Total simulation ticks since start.
    pub sim_tick: Tick,
}
