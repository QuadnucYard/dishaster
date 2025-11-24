use dishaster_views::DayHudState;

use crate::prelude::*;

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
    fn start(&mut self, commands: GuiCommands, _provider: AssetProvider) {
        let cmd = commands.clone();
        self.start_button.on_click.connect(move || {
            cmd.push_req(GameRequest::StartRun);
        });

        let cmd = commands.clone();
        self.dev_end_button.on_click.connect(move || {
            cmd.push_req(GameRequest::EndRun);
        });

        let cmd = commands.clone();
        self.exit_button.on_click.connect(move || {
            cmd.push_req(AppRequest::ExitLevel);
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
    }

    pub fn set_dev_enabled(&mut self, enabled: bool) {
        self.dev_end_button.set_visible(enabled);
        self.dev_end_button.set_enabled(enabled);
    }
}
