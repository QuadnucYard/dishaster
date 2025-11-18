//! Snapshot representations of dish display instances and agent appearances.

mod management;
mod param;
mod trial;

use dishrupt_core::prelude::*;
pub use management::*;
pub use param::*;
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
    /// Optional topic associated with this feedback (for trial triggering).
    pub topic: Option<FeedbackTopic>,
    /// Whether this feedback can trigger a trial (based on topic and trial corpus).
    pub can_trigger_trial: bool,
}

/// Content of feedback events.
#[derive(Debug, Clone)]
pub enum Feedback {
    /// Thought bubble with emoji or short text
    Thought(EcoString),
    /// Spoken feedback with implicit content
    Speech,
}

/// Different feedback topics that can be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackTopic {
    /// No dishes appealed to the diner
    Appeal,
    /// Queue too long / exceeded patience
    Queue,
    /// Missing tableware (tray or chopsticks)
    Tableware,
    /// Dish below expectation
    Quality,
    /// Pricing complaints
    Price,
    /// Food hygiene issues encountered
    Hygiene,
    /// Dish tastes bad
    Taste,
    /// Still hungry after meal
    Hunger,
    /// Positive feedback
    Praise,
    /// Special topic
    Crab,
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
    pub entries: Vec<Vec<String>>,
}

/// Reputation system state update for UI display
#[derive(Debug, Clone)]
pub struct ReputationView {
    /// Current reputation value [0, 100]
    pub reputation: f32,
    /// Reputation change in this update
    pub reputation_delta: f32,
    /// Food Safety Risk Index [0, 100]
    pub fsri: f32,
    /// Food quality level [0, 100]
    pub food_quality: f32,
}

/// Trial feedback impact on both diner psychology and reputation
///
/// Emitted when trial responses or timeouts affect game state,
/// allowing the trial GUI to display these consequences in real-time.
#[derive(Debug, Clone)]
pub struct TrialImpactView {
    /// Changes to diner's psychological state (if any)
    pub psych_impact: Option<PsychImpactView>,
    /// Changes to global reputation (if any)
    pub reputation_impact: Option<ReputationView>,
}

/// Impact on a diner's psychological state
#[derive(Debug, Clone)]
pub struct PsychImpactView {
    /// Change in mood [-100, 100]
    pub mood_delta: f32,
    /// Change in trust [-100, 100]
    pub trust_delta: f32,
    /// Change in patience [-100, 100]
    pub patience_delta: f32,
}

/// Type of game ending reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndingType {
    /// Bad ending: Reputation dropped to 0 (forced).
    BadReputation,
    /// Good ending: Reputation reached 100 (optional).
    GoodReputation,
    /// Bad ending: Food safety shutdown (forced).
    Rectification,
}

/// View data for an ending, including display information.
#[derive(Debug, Clone)]
pub struct EndingView {
    /// String identifier for this ending (for localization).
    pub id: EcoString,
    /// Whether the player can continue playing after this ending.
    pub can_continue: bool,
}
