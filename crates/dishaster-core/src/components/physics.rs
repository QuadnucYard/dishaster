use crate::prelude::*;

/// Axis-aligned bounding box collider for spatial collision detection
#[derive(Component, Deref, DerefMut)]
pub struct BoxCollider(pub dishaster_navigation::BoxCollider);
