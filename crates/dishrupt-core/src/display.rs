use std::num::NonZero;

use bevy_color::Color;
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
    pub proto: PrefabReference,

    pub color: Modified<Color>,

    pub visible: Modified<bool>,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            proto: PrefabReference::default(),
            color: Color::WHITE.into(),
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

    pub parent: Modified<Option<Entity>>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: 0.0,
            parent: None.into(),
        }
    }
}

impl Transform {
    pub fn snapshot(&mut self) -> TransformSnapshot {
        let parent = self
            .parent
            .clone()
            .map(|v| v.map(|e| EntityId(NonZero::new(e.to_bits()).unwrap())));
        self.parent.reset_modified();
        TransformSnapshot {
            position: self.position,
            scale: self.scale,
            rotation: self.rotation,
            parent,
        }
    }

    pub fn detach(&mut self) {
        *self.parent = None;
    }
}

#[derive(Debug)]
pub struct TransformSnapshot {
    pub position: Vec3,
    pub scale: Vec3,
    /// In radians
    pub rotation: f32,

    pub parent: Modified<Option<EntityId>>,
}

// ===

#[derive(Debug)]
pub struct DisplaySnapshot {
    pub core_id: EntityId,
    pub proto: PrefabReference,
    // pub display: DisplayState,
    pub transform: TransformSnapshot,
}
