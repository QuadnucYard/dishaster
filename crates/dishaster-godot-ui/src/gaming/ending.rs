use dishaster_views::EndingView;

use crate::prelude::*;

/// Ending screen showing game conclusion
#[derive(UITree)]
#[ui_tree]
pub struct EndingGui {
    #[child("%TitleLabel")]
    title_label: LabelA,

    #[child("%DescriptionLabel")]
    desc_label: LabelA,

    #[child("%ContinueButton")]
    continue_btn: ButtonA,

    #[child("%ExitButton")]
    exit_btn: ButtonA,
}

#[ui_tree_api]
impl UITree for EndingGui {}

impl Gui for EndingGui {
    fn start(&mut self, commands: GuiCommands) {
        // Continue button - returns to game (only for GoodReputation ending)
        let cmd = commands.clone();
        self.continue_btn.on_click.connect(move || {
            cmd.push_req(GameRequest::NextDay);
        });

        // Exit button - returns to main menu
        let cmd = commands.clone();
        self.exit_btn.on_click.connect(move || {
            cmd.push_req(AppRequest::ExitLevel);
        });
    }
}

impl EndingGui {
    /// Show ending screen with the given type
    pub fn show_ending(&mut self, ending: EndingView) {
        let id = ending.id;
        self.title_label.set_text(&tr!("ending--{}.title", id));
        self.desc_label.set_text(&tr!("ending--{}.desc", id));

        // Good ending: optional, show continue button; others: forced exit, no continue
        self.continue_btn.set_visible(ending.can_continue);

        self.show();
    }
}
