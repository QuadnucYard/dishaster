//! Simulation commands and control interfaces.

use crate::prelude::*;

/// Commands that can be sent to the simulation from external sources.
pub enum SimCommand {
    /// Start a new run (spawning diners, etc.)
    StartRun,
    /// Finish the current run immediately.
    EndRun,

    /// Request distance to a target point from the navigation grid.
    QueryDistance(Vec2),
    /// Request distance field data from the navigation grid.
    QueryDistances,
}
