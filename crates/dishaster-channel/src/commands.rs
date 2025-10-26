//! Simulation commands and control interfaces.

use dishaster_models::PricingMethod;
use dishrupt_core::{EntityId, prelude::*};

use crate::snapshots::DebugFlags;

/// Commands that can be sent to the simulation from external sources.
pub enum SimCommand {
    /// Set debug flags.
    SetDebugFlags(DebugFlags),

    /// Start a new run (spawning diners, etc.)
    StartRun,
    /// Finish the current run immediately.
    EndRun,

    /// Apply edited dish pricing before starting service.
    UpdateDishPricing {
        /// Entity ID of the dish being updated.
        dish_entity: EntityId,
        /// Updated pricing configuration selected by the player.
        pricing: PricingMethod,
    },

    /// Request distance to a target point from the navigation grid.
    QueryDistance(Vec2),
    /// Request distance field data from the navigation grid.
    QueryDistances,

    /// Start a trial for the given diner entity.
    TrialStart(EntityId),
    /// Launch the trial after intro is complete.
    TrialLaunch,
    /// Choose a keyword during the trial.
    TrialChooseKeyword(EcoString),
    /// Timeout the current trial response.
    TrialTimeout,
    /// Proceed to the next dialogue of the trial.
    TrialProceed,
}
