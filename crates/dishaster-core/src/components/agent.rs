use crate::prelude::*;

/// Runtime component for movement state and behavior
#[derive(Component)]
pub struct Movement {
    /// Current position in the canteen
    pub position: Vec2,
    /// Target position the agent is moving towards
    pub target_position: Vec2,
}
