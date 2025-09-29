use dishaster_navigation::NavPath;

use crate::prelude::*;

/// Runtime component for movement state and behavior
#[derive(Component)]
pub struct Movement {
    /// Base walking speed in meters per second.
    pub walking_speed: f32,
    /// Speed factor applied to this entity's base movement speed.
    pub speed_factor: f32,
    /// The radius of the agent for collision avoidance.
    pub radius: f32,
    /// How impatient the agent is (0.0 = patient, 1.0 = very impatient).
    pub impatience: f32,

    /// Whether the agent is currently moving
    pub is_moving: bool,
    /// Current position in the canteen
    pub pos: Vec2,
    /// The final destination the agent is moving towards.
    pub target_pos: Vec2,
    /// Current velocity vector
    pub velocity: Vec2,
    /// The calculated path to the target_pos.
    pub path: NavPath,
    /// The last tick when the path was calculated.
    pub last_path_tick: u32,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            walking_speed: 1.0,
            speed_factor: 1.0,
            radius: 0.0,
            impatience: 1.0,

            is_moving: false,
            pos: Vec2::ZERO,
            target_pos: Vec2::ZERO,
            velocity: Vec2::ZERO,
            path: Default::default(),
            last_path_tick: 0,
        }
    }
}
