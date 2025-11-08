//! Protocol for opening animation simulation events

use dishrupt_core::{EntityId, prelude::*};

/// Snapshot containing both display transforms and presentation data
pub struct Snapshot {
    /// Display snapshots for Stage positioning
    pub display: Vec<DisplaySnapshot>,
    /// Presentation data for continuous visual updates
    pub objects: Vec<ObjectSnapshot>,
}

/// Presentation data for visual effects that update every frame
#[derive(Clone)]
pub struct ObjectSnapshot {
    /// Entity ID
    pub entity: EntityId,
    /// Item type for presenter routing
    pub item_type: ItemType,
    /// Alpha/opacity (0.0-1.0) - mainly for text fade effects
    pub alpha: f32,
    /// Wave animation phase for text
    pub wave_phase: f32,
    /// Optional color tint (R, G, B)
    pub color: Option<(f32, f32, f32)>,
}

/// Type of opening animation item
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    /// Food emoji
    Food,
    /// Face emoji
    Face,
    /// Review text
    Text,
}
/// Events emitted by opening simulation
pub enum SimEvent {
    /// Food icon spawned with visual variant and color
    FoodSpawned {
        /// Entity ID of spawned food
        entity: EntityId,
        /// Sprite variant index (0-7)
        variant: u8,
        /// RGB color tint (0.0-1.0)
        color: (f32, f32, f32),
    },
    /// Face icon spawned with visual variant
    FaceSpawned {
        /// Entity ID of spawned face
        entity: EntityId,
        /// Sprite variant index (0-7)
        variant: u8,
    },
    /// Review text spawned with content
    TextSpawned {
        /// Entity ID of spawned text
        entity: EntityId,
        /// Text content to display
        content: String,
    },
    /// Any object despawned
    ObjectDespawned(EntityId),
}
