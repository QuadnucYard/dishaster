//! Collision detection and spatial algorithms
//!
//! Contains spatial hash grids and collision detection utilities
//! for efficient proximity queries and physics simulation.

use crate::prelude::*;

/// Axis-aligned bounding box collider for spatial collision detection
#[derive(Debug, Clone, Copy)]
pub struct BoxCollider {
    /// Center position of the collider
    pub center: Vec2,
    /// Width and height dimensions of the collider
    pub size: Vec2,
}

impl BoxCollider {
    /// Converts the collider to a Bevy Rect for intersection testing
    pub fn extent(&self) -> Rect {
        Rect::from_center_size(self.center, self.size)
    }
}
