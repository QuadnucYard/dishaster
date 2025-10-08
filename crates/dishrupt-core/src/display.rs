// use bevy_color::Color;
use serde::{Deserialize, Serialize};

use crate::{EntityId, asset::PrefabReference, prelude::*, utils::Modified};

// ===

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisplayModel {
    pub res: PrefabReference,

    #[serde(default)]
    pub scale: f32,
}

impl Default for DisplayModel {
    fn default() -> Self {
        Self {
            res: Default::default(),
            scale: 1.0,
        }
    }
}

impl DisplayModel {
    pub fn simple(res: impl Into<EcoString>) -> Self {
        Self {
            res: PrefabReference::new(res),
            scale: 1.0,
        }
    }

    pub fn referred(res: PrefabReference) -> Self {
        Self { res, scale: 1.0 }
    }
}

// ===

#[derive(Component)]
pub struct CoreId(pub EntityId);

// ===

#[derive(Component)]
pub struct DisplayState {
    /// Reference to the prefab resource
    pub proto: PrefabReference,
    /// Optional name override for the node.
    pub name: Option<EcoString>,

    // pub color: Modified<Color>,
    pub visible: Modified<bool>,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            proto: PrefabReference::default(),
            name: None,
            // color: Color::WHITE.into(),
            visible: true.into(),
        }
    }
}

// ===

#[derive(Component)]
pub struct Transform {
    pub position: Vec3,
    pub scale: Vec3,
    /// In radians
    pub rotation: f32,

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
    pub fn snapshot(&self) -> TransformSnapshot {
        TransformSnapshot {
            position: self.position,
            scale: self.scale,
            rotation: self.rotation,
            parent: self.parent.map(Into::into),
        }
    }

    pub fn detach(&mut self) {
        self.parent = None;
    }
}

#[derive(Debug)]
pub struct TransformSnapshot {
    pub position: Vec3,
    pub scale: Vec3,
    /// In radians
    pub rotation: f32,

    pub parent: Option<EntityId>,
}

// ===

#[derive(Debug)]
pub struct DisplaySnapshot {
    pub core_id: EntityId,
    pub proto: PrefabReference,
    pub name: Option<EcoString>,
    // pub display: DisplayState,
    pub transform: TransformSnapshot,
}
