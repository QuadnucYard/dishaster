use bevy_math::Rect;

use crate::prelude::*;

/// Axis-aligned bounding box collider for spatial collision detection
#[derive(Component, Debug, Clone, Copy)]
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
