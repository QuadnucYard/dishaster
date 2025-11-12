//! UI protocol definitions for Dishaster.

use dishaster_views::*;
use dishrupt_core::prelude::*;

/// Requests that can be sent to the overall application.
pub enum AppRequest {
    /// Quite the game application.
    Quit,

    /// Start a new game run at the current level.
    EnterLevel,
    /// Exit the current game run and return to the main menu.
    ExitLevel,

    /// Show credits screen.
    ShowCredits,
    /// Return to main menu from credits or other screens.
    BackToMenu,
}

impl UiRequest for AppRequest {}

/// Requests that can be sent from GUI to the in-game scene.
pub enum GameRequest {
    /// Start the current run (from preparation phase).
    StartRun,
    /// End the current run immediately.
    EndRun,
    /// Skip to the next day (from settlement phase).
    NextDay,

    /// Change the simulation ticks-per-second rate.
    SetTps(f32),
    /// Enable or disable debug mode.
    SetDebugMode(bool),

    /// Apply the player's chosen pricing to a dish slot.
    ApplyDishPrice {
        /// Entity ID of the dish to update.
        dish: EntityId,
        /// Pricing method to apply.
        method: PricingMethod,
    },

    /// Mark trial intro as done and proceed to main trial.
    TrialIntroDone,
    /// Check a keyword during trial interaction.
    TrialCheckKeyword(usize),
    /// Navigate back from thought display in trial.
    TrialBackFromThought,
    /// Submit a response choice during trial.
    TrialRespond(usize),
    /// Mark trial response as done and proceed.
    TrialResponseDone,
    /// Handle trial timeout.
    TrialTimeout,

    /// Confirm settlement phase and proceed to decision-making.
    ConfirmSettlement,
    /// Select a decision from the available options.
    SelectDecision(usize),
    /// Confirm incident notification and continue to preparation phase.
    ConfirmIncident,
}

impl UiRequest for GameRequest {}

/// Commands emitted from game logic to mutate UI state.
///
/// These are processed by the scene layer (`dishaster-godot`) which owns the UI.
/// Game logic (`dishaster-godot-game`) returns these commands instead of
/// directly mutating UI components.
pub enum UiCommand {
    /// Signal that the current run has finished and UI should transition.
    FinishRun,
    /// Signal that the current day has finished and should advance to next day.
    FinishDay,

    /// Update the displayed TPS value in the time stats UI.
    UpdateTpsDisplay(f32),
    /// Update the HUD to the supplied state.
    UpdateDayHud(Box<DayHudState>),
    /// Update the stats display.
    UpdateStats(Box<StatsView>),

    /// Request opening the dish price editor for a given dish entity.
    OpenDishPriceEditor(DishPriceView),

    /// Request refill for a dispenser.
    RefillDispenser(EntityId),

    /// Start a trial for the given diner entity.
    TrialStart(EntityId),
    /// Show trial intro.
    TrialIntro(Box<TrialIntro>),
    /// Trial diner speaks.
    TrialLeftSpeak(Box<TrialStatement>),
    /// Trial player responds.
    TrialRightSpeak(Box<TrialSpeech>),
    /// Trial has ended.
    TrialEnd,

    /// Show management decisions to the player.
    ShowDecisionSelection(Box<ManagementDecisionsView>),
    /// Show incident notification at day start (random incident auto-applied).
    ShowIncidentNotification(Box<ManagementIncidentView>),
    // /// Close incident/decision UI and return to normal flow.
    // CloseIncidentDecisionUI,
    /// Show a hint notification to the player.
    ShowHint {
        /// Localized hint message.
        message: String,
    },
}

#[allow(missing_docs)]
pub struct StatsView {
    pub sim_tick: u32,
    pub sim_time: f64,

    pub fps: f32,
    pub ups: f32,

    pub current_diners: usize,
    pub total_visits: usize,
    /// Number of diners who completed their meal today
    pub completed_diners: usize,

    /// Total revenue collected today
    pub revenue: f32,
    /// Total food consumed today in kilograms
    pub consumption_kg: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_size() {
        const MAX_SIZE: usize = 48;
        let actual_size = std::mem::size_of::<UiCommand>();
        assert!(
            actual_size <= MAX_SIZE,
            "UiCommand size is {}, exceeds maximum of {}",
            actual_size,
            MAX_SIZE
        );
    }
}
