//! Snapshot representations of dish display instances and agent appearances.

mod trial;

use dishrupt_core::prelude::*;
pub use trial::*;

/// Describes the information to display in the in-game day loop overlay during
/// active play (preparation or service phases).
pub struct DayHudState {
    /// Textual label showing which day is active.
    pub day_label: String,
    /// Description of the current phase (preparation or running).
    pub phase_label: String,
    /// Rich-text friendly body summarizing guidance or results.
    pub details: String,
    /// Whether the start/resume button should be shown.
    pub show_start: bool,
    /// Whether the start button is interactable.
    pub enable_start: bool,
    /// Whether the developer end-day button should be visible.
    pub show_dev: bool,
    /// Whether the developer end-day button is interactable.
    pub enable_dev: bool,
}

/// Snapshot of a dish display instance for presentation systems.
#[derive(Debug)]
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

/// Captures the information needed to populate the dish price editor popup.
#[derive(Clone)]
pub struct DishPriceView {
    /// Entity ID of the dish being edited.
    pub entity: EntityId,
    /// Name of the dish.
    pub dish_name: String,
    /// Original pricing method before editing.
    pub original_price: PricingMethod,
    /// Current pricing method being edited.
    pub current_price: PricingMethod,
}

/// Feedback emitted by core simulation systems for client presentation.
#[derive(Debug)]
pub struct FeedbackView {
    /// Entity currently expressing the feedback.
    pub entity: EntityId,
    /// Content of the feedback.
    pub content: Feedback,
}

/// Content of feedback events.
#[derive(Debug, Clone)]
pub enum Feedback {
    /// Thought bubble with emoji or short text
    Thought(EcoString),
    /// Spoken feedback with implicit content
    Speech,
}

/// Complete appearance configuration for an agent
#[derive(Debug)]
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
#[derive(Debug)]
pub struct BodyPart {
    /// Sprite variant index
    pub variant: SpriteVariant,
    /// Color transformation for this part
    pub color_transform: ColorTransform,
}

/// Complete credits information for display in the credits scene
pub struct CreditsView {
    /// List of credit sections
    pub sections: Vec<CreditSectionView>,
}

/// A single section in the credits
pub struct CreditSectionView {
    /// Localized title for the section
    pub title: String,
    /// List of names/entries in this section
    pub entries: Vec<String>,
}
