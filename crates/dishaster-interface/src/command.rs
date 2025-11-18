//! Simulation commands and control interfaces.

use dishaster_views::{FeedbackTopic, PricingMethod, SpeechId};
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

    /// Request refill for a dispenser.
    RefillDispenser(EntityId),

    /// Start a trial for the given diner entity with optional topic.
    TrialStart {
        /// The diner entity triggering the trial.
        diner: EntityId,
        /// Optional topic that triggered this trial (for filtering relevant speeches).
        topic: Option<FeedbackTopic>,
    },
    /// Launch the trial after intro is complete.
    TrialLaunch,
    /// Choose a response during the trial.
    TrialRespond(SpeechId),
    /// Timeout the current trial response.
    TrialTimeout,
    /// Request response candidates for a specific keyword (lazy loading).
    TrialRequestCandidates {
        /// Id of the current speech in the trial.
        speech_id: SpeechId,
        /// Index of the keyword within the current speech.
        keyword_index: usize,
    },
    /// Proceed to the next dialogue of the trial.
    TrialProceed,

    /// Apply a selected management decision by index.
    ApplyManagementDecision(usize),

    /// [DEV] Adjust reputation by a delta value.
    DevAdjustReputation(f32),
    /// [DEV] Trigger inspector visit.
    DevInspectorVisit(bool),
    /// [DEV] Trigger crab.
    DevCrab,
}
