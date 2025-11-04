//! Resources for opening animation simulation

use dishrupt_core::asset::PrefabReference;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Configuration for opening animation
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct OpeningConfig {
    /// World bounds (meters)
    pub world_bound: Rect,
    /// Spawn interval for dish icons (seconds)
    pub dish_spawn_interval: f32,
    /// Spawn interval for emoji icons (seconds)
    pub emoji_spawn_interval: f32,
    /// Spawn interval for review texts (seconds)
    pub text_spawn_interval: f32,
    /// Maximum number of dish icons
    pub max_dishes: usize,
    /// Maximum number of emoji icons
    pub max_emojis: usize,
    /// Maximum number of review texts
    pub max_texts: usize,
    /// Gravity acceleration (m/s²)
    pub gravity: f32,
}

impl Default for OpeningConfig {
    fn default() -> Self {
        // World centered at origin: [-10, 10] x [-6, 6]
        Self {
            world_bound: Rect::new(-10.0, -6.0, 10.0, 6.0),
            dish_spawn_interval: 0.2,
            emoji_spawn_interval: 0.3,
            text_spawn_interval: 4.0,
            max_dishes: 25,
            max_emojis: 8,
            max_texts: 4,
            gravity: 9.8,
        }
    }
}

/// Assets and configurable content for opening animation
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct OpeningAssets {
    /// Universal prefab for all dish icons (presenter will adjust appearance later)
    pub dish_prefab: PrefabReference,
    /// Universal prefab for all emoji icons (presenter will adjust appearance later)
    pub emoji_prefab: PrefabReference,
    /// Prefab used for review text display (label prefab)
    pub text_prefab: PrefabReference,
    /// Review text corpus to display
    pub review_texts: Vec<String>,
}

impl Default for OpeningAssets {
    fn default() -> Self {
        Self {
            dish_prefab: PrefabReference::new("opening/dish"),
            emoji_prefab: PrefabReference::new("opening/emoji"),
            text_prefab: PrefabReference::new("opening/text"),
            review_texts: vec![
                "此食堂当为世界第八大奇迹！".into(),
                "吃完感觉灵魂得到了升华！".into(),
                "建议直接列入非物质文化遗产".into(),
                "食材新鲜度：刚从侏罗纪挖出来".into(),
                "厨师是不是在用爱发电？".into(),
                "食堂教会我，生活就是一场赌博".into(),
                "在排队中思考人生的意义".into(),
                "薛定谔的食堂：打开饭盒前，什么都有可能".into(),
                "吃完后我看见了第五维度".into(),
                "每个窗口都通往不同的平行宇宙".into(),
            ],
        }
    }
}

/// Spawn timers resource
#[derive(Resource, Debug, Default)]
pub struct SpawnTimers {
    /// Timer for dish spawning
    pub dish: f32,
    /// Timer for emoji spawning
    pub emoji: f32,
    /// Timer for text spawning
    pub text: f32,
}

/// Simple time resource for delta-time updates
#[derive(Resource, Debug)]
pub struct DeltaTime {
    /// Delta time in seconds
    pub delta: f32,
}

impl Default for DeltaTime {
    fn default() -> Self {
        Self { delta: 0.0 }
    }
}
