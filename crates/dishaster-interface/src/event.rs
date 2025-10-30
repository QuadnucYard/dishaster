//! Presentation events emitted by the core simulation for client display.

use dishaster_views::{
    Appearance, DishView, FeedbackView, TrialIntro, TrialSpeech, TrialStatement,
};
use dishrupt_core::EntityId;

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
    DishSpawned(DishView),
    /// Agent feedback.
    Feedback(FeedbackView),

    /// Show trial intro.
    TrialIntro(TrialIntro),
    /// Trial diner speaks.
    TrialLeftSpeak(TrialStatement),
    /// Trial player responds.
    TrialRightSpeak(TrialSpeech),
    /// Trial has ended.
    TrialEnd,
}
