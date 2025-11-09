//! Presentation events emitted by the core simulation for client display.

use dishaster_views::{
    Appearance, DishView, FeedbackView, ManagementDecisionsView, ManagementIncidentView,
    TrialIntro, TrialSpeech, TrialStatement,
};
use dishrupt_core::{EntityId, prelude::EcoString};

/// Presentation events emitted by the core simulation for client display.
pub enum SimEvent {
    /// The current day has completed (all diners have exited and time limit reached).
    DayCompleted,

    /// A dispenser has spawned.
    DispenserSpawned(EntityId),
    /// Dispenser stock has changed.
    DispenserStockChanged {
        /// Dispenser entity ID.
        entity: EntityId,
        /// Current stock amount.
        current_stock: u32,
        /// Maximum capacity.
        capacity: u32,
    },
    /// A dish has spawned with its pricing snapshot.
    DishSpawned(DishView),

    /// An agent has spawned.
    AgentSpawned {
        /// Spawned agent entity ID.
        entity: EntityId,
        /// Optional appearance customization data.
        appearance: Option<Box<Appearance>>,
    },
    /// An agent has despawned from the simulation.
    AgentDespawned(EntityId),
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
    TrialIntro(Box<TrialIntro>),
    /// Trial diner speaks.
    TrialLeftSpeak(Box<TrialStatement>),
    /// Trial player responds.
    TrialRightSpeak(Box<TrialSpeech>),
    /// Trial has ended.
    TrialEnd,

    /// Show management decisions to the player.
    ShowManagementDecisions(Box<ManagementDecisionsView>),
    /// Show incident notification at day start.
    ShowManagementIncident(Box<ManagementIncidentView>),

    /// Show a hint to the player for first-time events.
    ShowHint(EcoString),
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
