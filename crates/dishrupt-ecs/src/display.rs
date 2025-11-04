//! Display-related ECS components.

use dishrupt_core::prelude::*;

use crate::prelude::*;

/// Root entity for all display-related objects in the scene
#[derive(Resource)]
pub struct DisplayRoot(pub Entity);

/// Display state component for display entities
#[derive(Component, Default)]
pub struct DisplayState {
    /// Reference to the prefab resource
    pub proto: PrefabReference,
    /// Optional name override for the node.
    pub name: Option<EcoString>,
}

/// Transform component for display entities
#[derive(Component)]
pub struct Transform {
    /// Position in 3D space
    pub position: Vec3,
    /// Scale in 3D space
    pub scale: Vec3,
    /// Rotation around Z axis, in radians
    pub rotation: f32,

    /// Optional parent entity
    pub parent: Option<Entity>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: 0.0,
            parent: None,
        }
    }
}

impl Transform {
    /// Create a snapshot of this transform's current state
    pub fn snapshot(&self) -> TransformSnapshot {
        TransformSnapshot {
            position: self.position,
            scale: self.scale,
            rotation: self.rotation,
            parent: self.parent.map(ToEntityId::to_entity_id),
        }
    }

    /// Detach this transform from its parent
    pub fn detach(&mut self) {
        self.parent = None;
    }
}
