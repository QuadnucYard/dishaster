//! Presentation events emitted by the core simulation for client display.

use dishaster_models::{
    Appearance, ModelId, PricingMethod, TrialIntro, TrialSpeech, TrialStatement,
};
use dishrupt_core::{EntityId, prelude::*};

/// Presentation events emitted by the core simulation for client display.
pub enum SimEvent {
    /// The current day has completed (all diners have exited and time limit reached).
    DayCompleted,
    /// An agent has spawned in the simulation.
    AgentSpawned {
        /// Spawned agent entity ID.
        entity: EntityId,
        /// Optional appearance customization data.
        appearance: Option<Appearance>,
    },
    /// An agent has despawned from the simulation.
    AgentDespawned(EntityId),
    /// A dish has spawned in the simulation with its pricing snapshot.
    DishSpawned(DishViewModel),
    /// Agent feedback.
    Feedback(FeedbackEvent),

    /// Show trial intro.
    TrialIntro(TrialIntro),
    /// Trial diner speaks.
    TrialLeftSpeak(TrialStatement),
    /// Trial player responds.
    TrialRightSpeak(TrialSpeech),
    /// Trial has ended.
    TrialEnd,
}

/// Snapshot of a dish display instance for presentation systems.
#[derive(Debug, Clone)]
pub struct DishViewModel {
    /// Display entity backing this dish presentation.
    pub entity: EntityId,
    /// Dish model identifier for lookup and metadata.
    pub dish_id: ModelId,
    /// Current pricing configuration applied to this slot.
    pub pricing: PricingMethod,
}

/// Feedback emitted by core simulation systems for client presentation.
#[derive(Debug, Clone)]
pub struct FeedbackEvent {
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
