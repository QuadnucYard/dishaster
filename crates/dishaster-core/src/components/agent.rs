use crate::prelude::*;

/// Runtime component for movement state and behavior
#[derive(Component, Default)]
pub struct Movement {
    /// Current position in the canteen
    pub pos: Vec2,
    /// The final destination the agent is moving towards.
    pub target_pos: Vec2,
    /// The next immediate waypoint in the path.
    pub next_waypoint: Vec2,
    /// Current velocity vector
    pub velocity: Vec2,
    /// The calculated path to the target_pos.
    pub path: Vec<Vec2>,
    /// Position in the previous tick, used for interpolation.
    pub last_pos: Vec2,
    /// When true, this agent should be ignored by collision avoidance while it finds a fallback path.
    pub ignoring_collisions: bool,
}
