//! Components for opening animation items

use dishrupt_core::asset::PrefabReference;

use crate::prelude::*;

/// Component for dish icon entity. Holds the prefab reference for this dish.
#[derive(Component)]
pub struct DishIcon {
    /// Prefab reference used by the presenter / stage
    pub proto: PrefabReference,
}

/// Component for emoji icon entity. Holds the prefab reference for this emoji.
#[derive(Component)]
pub struct EmojiIcon {
    /// Prefab reference used by the presenter / stage
    pub proto: PrefabReference,
}

// (DishIcon and EmojiIcon are defined above with prefab references)

/// Component for review text entity
#[derive(Component)]
pub struct ReviewText {
    /// The text content
    pub content: String,
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
