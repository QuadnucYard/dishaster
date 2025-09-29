use dishaster_navigation::NavPath;

use crate::prelude::*;

/// Runtime component for movement state and behavior
#[derive(Component)]
pub struct Movement {
    /// Current position in the canteen
    pub pos: Vec2,
    /// The final destination the agent is moving towards.
    pub target_pos: Vec2,
    /// Base walking speed in meters per second.
    pub walking_speed: f32,
    /// Speed factor applied to this entity's base movement speed.
    pub speed_factor: f32,
    /// Current velocity vector
    pub velocity: Vec2,
    /// The calculated path to the target_pos.
    pub path: NavPath,
    /// The next immediate waypoint in the path.
    pub next_waypoint: Vec2,
    /// The radius of the agent for collision avoidance.
    pub radius: f32,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            target_pos: Vec2::ZERO,
            walking_speed: 1.0,
            speed_factor: 1.0,
            velocity: Vec2::ZERO,
            path: Default::default(),
            next_waypoint: Vec2::ZERO,
            radius: 0.0,
        }
    }
}
