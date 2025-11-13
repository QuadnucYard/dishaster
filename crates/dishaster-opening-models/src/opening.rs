use crate::prelude::*;

/// Configuration for the Dishaster opening animation
#[derive(Debug, Clone, Deserialize)]
pub struct OpeningConfig {
    /// World configuration
    pub world: OpeningWorldConfig,
    /// Assets and configurable content
    pub assets: OpeningAssets,
}

/// Configuration for opening animation
#[derive(Debug, Clone, Deserialize)]
pub struct OpeningWorldConfig {
    /// World bounds (meters)
    pub world_bound: Rect,
    /// Spawn interval for food icons (seconds)
    pub food_spawn_interval: f32,
    /// Spawn interval for face icons (seconds)
    pub face_spawn_interval: f32,
    /// Spawn interval for review texts (seconds)
    pub text_spawn_interval: f32,
    /// Maximum number of food icons
    pub max_foods: usize,
    /// Maximum number of face icons
    pub max_faces: usize,
    /// Maximum number of review texts
    pub max_texts: usize,
    /// Gravity acceleration (m/s²)
    pub gravity: f32,
}

/// Assets and configurable content for opening animation
#[derive(Debug, Clone, Deserialize)]
pub struct OpeningAssets {
    /// Universal prefab for all foods (presenter will adjust appearance later)
    pub food_prefab: PrefabRef,
    /// Universal prefab for all faces (presenter will adjust appearance later)
    pub face_prefab: PrefabRef,
    /// Prefab used for review text display (label prefab)
    pub text_prefab: PrefabRef,
    /// Number of food sprite variants available
    pub food_variant_count: u8,
    /// Number of face sprite variants available
    pub face_variant_count: u8,
    /// Review text corpus to display
    pub review_texts: Vec<String>,
}
