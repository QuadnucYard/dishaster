//! Queries from the client to the simulation.

use dishrupt_core::prelude::*;

/// Queries that can be sent to the simulation from the client, without state mutation.
pub enum SimQuery {
    /// Request distance to a target point from the navigation grid.
    Distance(Vec2),
    /// Request distance field data from the navigation grid.
    Distances,

    /// Query feedback statistics for debugging purposes.
    FeedbackStats,
}
