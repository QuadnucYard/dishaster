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
    /// Diner's held items have changed.
    DinerItemsChanged {
        /// Diner entity ID.
        entity: EntityId,
        /// Type of change to the diner's items.
        change: DinerItemsChange,
    },
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

/// Types of changes to a diner's held items.
pub enum DinerItemsChange {
    /// Diner picked up a tray.
    PickTray(EntityId),
    /// Diner picked up chopsticks.
    PickChopsticks(EntityId),
    /// Diner picked up a dish.
    PickDish(EntityId),
    /// Diner started eating at a table.
    StartEating,
    /// Diner finished eating.
    FinishEating,
    /// Diner dropped all items (when returning dishes).
    DropAll,
}
