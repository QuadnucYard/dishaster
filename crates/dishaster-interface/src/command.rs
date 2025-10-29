//! Simulation commands and control interfaces.

use dishaster_models::PricingMethod;
use dishrupt_core::EntityId;

use crate::snapshots::DebugFlags;

/// Commands that can be sent to the simulation from the client that may mutate the state.
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

    /// Start a trial for the given diner entity.
    TrialStart(EntityId),
    /// Launch the trial after intro is complete.
    TrialLaunch,
    /// Choose a response during the trial.
    TrialRespond(usize),
    /// Timeout the current trial response.
    TrialTimeout,
    /// Proceed to the next dialogue of the trial.
    TrialProceed,
}
