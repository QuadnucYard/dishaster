use bevy_color::Color;
use serde::Deserialize;

use crate::{EntityId, asset::PrefabReference, prelude::*, utils::Modified};

// ===

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DisplayModel {
    pub res: PrefabReference,

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

pub struct DisplayState {
    pub color: Modified<Color>,

    pub visible: Modified<bool>,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            color: Color::WHITE.into(),
            visible: true.into(),
        }
    }
}

// ===

#[derive(Component)]
pub struct TransformState {
    pub position: Vec3,
    pub scale: Vec3,
    /// In radians
    pub rotation: f32,

    pub parent: Modified<Option<Entity>>,
}

impl Default for TransformState {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: 0.0,
            parent: None.into(),
        }
    }
}

impl TransformState {
    pub fn detach(&mut self) {
        *self.parent = None;
    }
}

pub struct TransformSnapshot {
    pub position: Vec3,
    pub scale: Vec3,
    /// In radians
    pub rotation: f32,

    pub parent: Modified<Option<EntityId>>,
}

// ===

pub struct DisplayBundle {
    pub display: DisplayState,
    pub transform: TransformState,
}

pub struct DisplaySnapshot {
    pub core_id: EntityId,
    pub proto: PrefabReference,
    // pub display: DisplayState,
    pub transform: TransformSnapshot,
}
