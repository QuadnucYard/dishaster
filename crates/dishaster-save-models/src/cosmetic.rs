//! Cosmetic appearance system for agents
//!
//! Defines customizable visual appearance with randomizable parts and colors.

use dishrupt_core::{asset::SpriteVariant, display::ColorTransform};

use crate::prelude::*;

/// Complete appearance configuration for an agent
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Appearance {
    /// Head/face sprite variant and color
    pub head: BodyPart,
    /// Upper garment (shirt, jacket, etc.)
    pub upper_garment: BodyPart,
    /// Lower garment (pants, skirt, etc.)
    pub lower_garment: BodyPart,
    /// Hand/arm appearance
    pub hands: BodyPart,
    /// Footwear
    pub shoes: BodyPart,
}

/// A single body part with its sprite variant and color transformation
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BodyPart {
    /// Sprite variant index
    pub variant: SpriteVariant,
    /// Color transformation for this part
    pub color_transform: ColorTransform,
}
