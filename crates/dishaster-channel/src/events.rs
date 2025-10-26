//! Presentation events emitted by the core simulation for client display.

use dishaster_models::{ModelId, PricingMethod, TrialIntro, TrialSpeech};
use dishrupt_core::{EntityId, prelude::*};

/// Presentation events emitted by the core simulation for client display.
pub enum PresentationEvent {
    /// The current day has completed (all diners have exited and time limit reached).
    DayCompleted,
    /// An agent has spawned in the simulation.
    AgentSpawned(EntityId),
    /// An agent has despawned from the simulation.
    AgentDespawned(EntityId),
    /// A dish has spawned in the simulation with its pricing snapshot.
    DishSpawned(DishViewModel),
    /// Agent feedback.
    Feedback(FeedbackEvent),

    /// Response to a distance query.
    QueryDistanceResponse(Option<f32>),
    /// Response to a distance field query.
    QueryDistancesResponse(QueryDistancesResponse),

    /// Show trial intro.
    TrialIntro(TrialIntro),
    /// Trial diner speaks.
    TrialLeftSpeak(TrialSpeech),
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

/// Response to a distance field query.
#[derive(Debug, Clone)]
pub struct QueryDistancesResponse {
    /// Width of the grid in cells.
    pub width: usize,
    /// Height of the grid in cells.
    pub height: usize,
    /// Size of each cell in world units.
    pub cell_size: f32,
    /// Flattened col-major grid of distances.
    pub data: Vec<f32>,
}
