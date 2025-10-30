//! Cosmetic appearance system for agents
//!
//! Defines customizable visual appearance with randomizable parts and colors.

use dishrupt_core::{asset::SpriteVariant, display::ColorTransform};

use super::prelude::*;

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

/// Ranges for randomizing appearance parts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceRanges {
    /// Number of available head variants
    pub head_variants: u8,
    /// Number of available upper garment variants
    pub upper_garment_variants: u8,
    /// Number of available lower garment variants
    pub lower_garment_variants: u8,
    /// Number of available hand variants
    pub hand_variants: u8,
    /// Number of available shoe variants
    pub shoe_variants: u8,
}

impl Default for AppearanceRanges {
    fn default() -> Self {
        Self {
            head_variants: 4,
            upper_garment_variants: 5,
            lower_garment_variants: 4,
            hand_variants: 3,
            shoe_variants: 3,
        }
    }
}
