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
}

impl UiRequest for AppRequest {}

/// Requests that can be sent to the in-game scene.
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
}

impl UiRequest for GameRequest {}

/// Commands emitted from game logic to mutate UI state.
///
/// These are processed by the scene layer (`dishaster-godot`) which owns the UI.
/// Game logic (`dishaster-godot-game`) returns these commands instead of
/// directly mutating UI components.
pub enum UiCommand {
    /// Signal that the current day has finished and UI should transition.
    FinishDay,

    /// Update the displayed TPS value in the time stats UI.
    UpdateTpsDisplay(f32),
    /// Update the HUD to the supplied state.
    UpdateDayHud(DayHudState),
    /// Update the stats display.
    UpdateStats(StatsView),

    /// Request opening the dish price editor for a given dish entity.
    OpenDishPriceEditor(DishPriceView),

    /// Request refill for a dispenser.
    RefillDispenser(EntityId),

    /// Start a trial for the given diner entity.
    TrialStart(EntityId),
    /// Show trial intro.
    TrialIntro(TrialIntro),
    /// Trial diner speaks.
    TrialLeftSpeak(TrialStatement),
    /// Trial player responds.
    TrialRightSpeak(TrialSpeech),
    /// Trial has ended.
    TrialEnd,
}

#[allow(missing_docs)]
pub struct StatsView {
    pub sim_tick: u32,
    pub sim_time: f64,

    pub fps: f32,
    pub ups: f32,

    pub current_diners: usize,
    pub total_visits: usize,
}
