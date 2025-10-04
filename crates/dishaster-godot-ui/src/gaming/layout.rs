use crate::{
    prelude::*,
    req::{EndDayRequest, ExitLevelRequest, StartRunRequest},
};

/// Describes the information to display in the in-game day loop overlay during
/// active play (preparation or service phases).
pub struct DayHudState {
    /// Textual label showing which day is active.
    pub day_label: String,
    /// Description of the current phase (preparation or running).
    pub phase_label: String,
    /// Rich-text friendly body summarizing guidance or results.
    pub details: String,
    /// Whether the start/resume button should be shown.
    pub show_start: bool,
    /// Whether the start button is interactable.
    pub enable_start: bool,
    /// Whether the developer end-day button should be visible.
    pub show_dev: bool,
    /// Whether the developer end-day button is interactable.
    pub enable_dev: bool,
}

#[derive(UITree)]
#[ui_tree]
pub struct GamingLayout {
    #[child("%DayLabel")]
    day_label: LabelA,
    #[child("%PhaseLabel")]
    phase_label: LabelA,
    #[child("%DetailsLabel")]
    details_label: RichLabelA,
    #[child("%StartButton")]
    start_button: ButtonA,
    #[child("%EndButton")]
    dev_end_button: ButtonA,
    #[child("%Exit")]
    exit_button: ButtonA,
}

#[ui_tree_api]
impl UITree for GamingLayout {}

impl Gui for GamingLayout {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.start_button.on_click.connect(move || {
            cmd.push_req(StartRunRequest);
        });

        let cmd = commands.clone();
        self.dev_end_button.on_click.connect(move || {
            cmd.push_req(EndDayRequest);
        });

        let cmd = commands.clone();
        self.exit_button.on_click.connect(move || {
            cmd.push_req(ExitLevelRequest);
        });
    }
}

impl GamingLayout {
    /// Update the HUD to match the supplied state.
    pub fn apply_state(&mut self, state: &DayHudState) {
        self.day_label.set_text(&state.day_label);
        self.phase_label.set_text(&state.phase_label);
        self.details_label.set_text(&state.details);

        self.start_button.set_visible(state.show_start);
        self.start_button
            .set_enabled(state.enable_start && state.show_start);

        self.dev_end_button.set_visible(state.show_dev);
        self.dev_end_button
            .set_enabled(state.enable_dev && state.show_dev);
    }
}
