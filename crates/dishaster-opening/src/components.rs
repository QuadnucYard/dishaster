//! Components for opening animation items

use crate::prelude::*;

/// Component for food icon entity
#[derive(Component)]
pub struct FoodObject {}

/// Component for face icon entity
#[derive(Component)]
pub struct FaceObject {}

/// Component for review text entity
#[derive(Component)]
pub struct TextObject {}

/// Color modulation for tinting sprites
#[derive(Component, Clone, Copy)]
pub struct ColorTint {
    /// Red channel (0.0 to 1.0)
    pub r: f32,
    /// Green channel (0.0 to 1.0)
    pub g: f32,
    /// Blue channel (0.0 to 1.0)
    pub b: f32,
}

impl ColorTint {
    /// Create white (no tint)
    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }

    /// Create random bright color
    pub fn random_bright(rng: &mut WorldRng) -> Self {
        Self {
            r: rng.random_range(0.7..1.0),
            g: rng.random_range(0.7..1.0),
            b: rng.random_range(0.7..1.0),
        }
    }
}

/// Position in simulation space (meters)
#[derive(Component)]
pub struct Position(pub Vec2);

/// Velocity vector (m/s)
#[derive(Component)]
pub struct Velocity(pub Vec2);

/// Rotation angle (radians)
#[derive(Component)]
pub struct Rotation(pub f32);

/// Rotation speed (radians/s)
#[derive(Component)]
pub struct RotationSpeed(pub f32);

/// Scale factor
#[derive(Component)]
pub struct Scale(pub f32);

/// Opacity (0.0 to 1.0) for text items
#[derive(Component)]
pub struct Alpha(pub f32);

/// Vertical falling speed (m/s) for text items
#[derive(Component)]
pub struct FallSpeed(pub f32);

/// Wave animation phase for text items
#[derive(Component)]
pub struct WavePhase(pub f32);
