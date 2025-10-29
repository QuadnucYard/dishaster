//! Cosmetic appearance system for agents
//!
//! Defines customizable visual appearance with randomizable parts and colors.

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

/// Sprite variant index for a body part
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

/// Color transformation applied to sprites
///
/// Uses HSV color space for intuitive adjustments.
/// The shader will apply these transforms to recolor sprites.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColorTransform {
    /// Hue shift in degrees (0-360, wraps around)
    pub hue_shift: f32,
    /// Saturation multiplier (0 = grayscale, 1 = original, >1 = more saturated)
    pub saturation: f32,
    /// Value/brightness multiplier (0 = black, 1 = original, >1 = brighter)
    pub value: f32,
    /// Alpha/transparency (0 = fully transparent, 1 = fully opaque)
    pub alpha: f32,
}

impl Default for ColorTransform {
    fn default() -> Self {
        Self {
            hue_shift: 0.0,
            saturation: 1.0,
            value: 1.0,
            alpha: 1.0,
        }
    }
}

impl ColorTransform {
    /// Create a color transform with no modifications
    pub fn identity() -> Self {
        Self::default()
    }
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
