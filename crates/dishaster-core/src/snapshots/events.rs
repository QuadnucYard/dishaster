use dishrupt_core::{EntityId, prelude::*};

/// Presentation events emitted by the core simulation for client display.
pub enum PresentationEvent {
    /// The current day has completed (all diners have exited and time limit reached).
    DayCompleted,
    /// An agent has spawned in the simulation.
    AgentSpawned(EntityId),
    /// An agent has despawned from the simulation.
    AgentDespawned(EntityId),
    /// Agent feedback.
    Feedback(FeedbackEvent),
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
