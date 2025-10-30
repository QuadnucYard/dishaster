//! Snapshot representations of dish display instances and agent appearances.

mod trial;

use dishrupt_core::prelude::*;
pub use trial::*;

/// Snapshot of a dish display instance for presentation systems.
#[derive(Debug, Clone)]
pub struct DishView {
    /// Display entity backing this dish presentation.
    pub entity: EntityId,
    /// Dish model identifier for lookup and metadata.
    pub dish_id: ModelId,
    /// Current pricing configuration applied to this slot.
    pub pricing: PricingMethod,
}

/// Different pricing strategies for dishes
#[derive(Debug, Clone, Copy)]
pub enum PricingMethod {
    /// Fixed price per serving
    PerPortion(f32),
    /// Price calculated by weight (per kg)
    ByWeight(f32),
}

/// Feedback emitted by core simulation systems for client presentation.
#[derive(Debug, Clone)]
pub struct FeedbackView {
    /// Entity currently expressing the feedback.
    pub entity: EntityId,
    /// Content of the feedback.
    pub content: Feedback,
    /// Simulation timestamp when the feedback was generated (seconds).
    pub timestamp: f64,
}

/// Content of feedback events.
#[derive(Debug, Clone)]
pub enum Feedback {
    /// Quiet thought bubble.
    Thought(EcoString),
    /// Spoken feedback with implicit content.
    Speech,
}

/// Complete appearance configuration for an agent
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct BodyPart {
    /// Sprite variant index
    pub variant: SpriteVariant,
    /// Color transformation for this part
    pub color_transform: ColorTransform,
}
