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
    /// Create a BoxCollider from a Bevy Rect
    pub fn from_rect(rect: Rect) -> Self {
        Self {
            center: rect.center(),
            size: rect.size(),
        }
    }

    /// Create a BoxCollider from center position and size
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        Self { center, size }
    }

    /// Converts the collider to a Bevy Rect for intersection testing
    pub fn extent(&self) -> Rect {
        Rect::from_center_size(self.center, self.size)
    }
}

impl From<Rect> for BoxCollider {
    fn from(value: Rect) -> Self {
        Self::from_rect(value)
    }
}
