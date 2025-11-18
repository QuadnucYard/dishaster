//! Presentation events emitted by the core simulation for client display.

use dishaster_views::{
    Appearance, DishView, EndingView, FeedbackView, InspectorResultView, ManagementDecisionsView,
    ManagementIncidentView, PricingMethod, ReputationView, TrialImpactView, TrialIntro,
    TrialResponseOption, TrialStatement,
};
use dishrupt_core::{EntityId, prelude::EcoString};

/// Presentation events emitted by the core simulation for client display.
#[derive(Debug)]
pub enum SimEvent {
    /// Request to persist player progress.
    Persist,

    /// The current run has completed (all diners have exited and time limit reached).
    RunCompleted,
    /// The current day has completed (decision made).
    DayCompleted,

    /// Reputation system state has been updated.
    ReputationUpdate(Box<ReputationView>),

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
    /// A dish's price has changed.
    DishPriceChanged {
        /// Dish entity ID.
        entity: EntityId,
        /// New pricing method.
        new_pricing: PricingMethod,
    },

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

    /// Show slogan at day start.
    ShowSlogan,
    /// Show crab at day start.
    ShowCrab,

    /// Show trial intro.
    TrialIntro(Box<TrialIntro>),
    /// Trial diner speaks.
    TrialLeftSpeak(Box<TrialStatement>),
    /// Trial player responds.
    TrialRightSpeak(Box<TrialStatement>),
    /// Response candidates for a keyword (lazy loaded).
    TrialResponseCandidates(Vec<TrialResponseOption>),
    /// Trial feedback impact on diner psychology and reputation.
    TrialImpact(Box<TrialImpactView>),
    /// Trial has ended.
    TrialEnd {
        /// Whether the trial ended due to a timeout.
        timeout: bool,
    },

    /// Show management decisions to the player.
    ShowManagementDecisions(Box<ManagementDecisionsView>),
    /// Show incident notification at day start.
    ShowManagementIncident(Box<ManagementIncidentView>),
    /// Show inspector visit result.
    ShowInspectorResult(Box<InspectorResultView>),

    /// Show tutorial dialog on first day.
    ShowTutorial,
    /// Show ending screen.
    ShowEnding(Box<EndingView>),

    /// Show a hint to the player for first-time events.
    ShowHint {
        /// Unique identifier for the hint.
        id: EcoString,
        /// Emission condition for the hint.
        condition: HintCondition,
    },
}

/// Types of changes to a diner's held items.
#[derive(Debug)]
pub enum DinerItemsChange {
    /// Diner picked up a tray.
    PickTray(EntityId),
    /// Diner picked up chopsticks.
    PickChopsticks(EntityId),
    /// Diner picked up a dish.
    PickDish(EntityId),
    /// Diner started eating at a table.
    StartEating(EntityId, usize),
    /// Diner finished eating.
    FinishEating,
    /// Diner dropped all items (when returning dishes).
    DropAll,
}

/// Emission condition for hints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintCondition {
    /// Always show the hint (e.g., critical warnings)
    Always,
    /// Show once per game profile (first-time tutorial hints)
    OnceGlobal,
    /// Show once per day (daily reminders)
    OnceLocal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_size() {
        const MAX_SIZE: usize = 40;
        let actual_size = std::mem::size_of::<SimEvent>();
        assert!(
            actual_size <= MAX_SIZE,
            "SimEvent size is {}, exceeds maximum of {}",
            actual_size,
            MAX_SIZE
        );
    }
}
