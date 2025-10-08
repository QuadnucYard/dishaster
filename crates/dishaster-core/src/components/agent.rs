use dishaster_navigation::NavPath;

use crate::prelude::*;

/// Marker component for identifying agent entities
#[derive(Component)]
pub struct AgentTag;

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

    /// Current position in the canteen
    pub pos: Vec2,
    /// Current velocity vector
    pub velocity: Vec2,
    /// The calculated path to the target_pos.
    pub path: NavPath,
    /// The target position to move towards.
    pub pending_target: Option<Vec2>,
    /// The last tick when the path was calculated. It is set when a new path is assigned.
    pub last_path_tick: Tick,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            walking_speed: 1.0,
            speed_factor: 1.0,
            radius: 0.0,
            impatience: 1.0,

            pos: Vec2::ZERO,
            velocity: Vec2::ZERO,
            path: Default::default(),
            pending_target: None,
            last_path_tick: 0,
        }
    }
}

impl Movement {
    /// Returns true if the agent has a path to follow, or is en route to a target.
    pub fn has_path(&self) -> bool {
        !self.path.is_empty() || self.pending_target.is_some()
    }

    /// Clear the current path and stop movement.
    pub fn stop(&mut self) {
        self.path.clear();
        self.pending_target = None;
        self.velocity = Vec2::ZERO;
    }
}
