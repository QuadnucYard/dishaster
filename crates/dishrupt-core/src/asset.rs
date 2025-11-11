use core::fmt;

use ecow::EcoString;
use serde::{Deserialize, Serialize};

/// Reference to a prefab resource.
///
/// The path is relative to `res://assets/{prefabs}/` in Godot.
/// It does not include file extension.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct PrefabReference(EcoString);

impl PrefabReference {
    /// Create a new prefab reference
    pub fn new(path: impl Into<EcoString>) -> Self {
        Self(path.into())
    }

    /// Get the prefab path
    pub fn path(&self) -> &EcoString {
        &self.0
    }
}

impl fmt::Display for PrefabReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Reference to a sprite resource
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Deserialize)]
pub struct SpriteReference(EcoString);

impl SpriteReference {
    /// Create a new sprite reference
    pub fn new(path: impl Into<EcoString>) -> Self {
        Self(path.into())
    }

    /// Get the sprite path
    pub fn path(&self) -> &EcoString {
        &self.0
    }
}

impl fmt::Display for SpriteReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Reference to an audio resource
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Deserialize)]
pub struct AudioRef(EcoString);

impl AudioRef {
    /// Create a new audio reference
    pub fn new(path: impl Into<EcoString>) -> Self {
        Self(path.into())
    }

    /// Get the audio path
    pub fn path(&self) -> &EcoString {
        &self.0
    }
}

impl fmt::Display for AudioRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Sprite variant index
///
/// Each part type has multiple sprite options (e.g., head_01, head_02, etc.)
/// This stores which variant to use.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteVariant(u8);

impl SpriteVariant {
    /// Create a new sprite variant
    pub fn new(index: u8) -> Self {
        Self(index)
    }

    /// Get the variant index
    pub fn index(self) -> u8 {
        self.0
    }
}
