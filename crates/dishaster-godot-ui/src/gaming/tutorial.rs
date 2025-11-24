use crate::prelude::*;

/// GUI for displaying tutorial dialog on first day
#[derive(UITree)]
#[ui_tree]
pub struct TutorialGui {
    #[child("%ConfirmButton")]
    confirm_button: ButtonA,
}

#[ui_tree_api]
impl UITree for TutorialGui {}

impl Gui for TutorialGui {
    fn start(&mut self, commands: GuiCommands, _provider: AssetProvider) {
        let cmd = commands.clone();
        self.confirm_button.on_click.connect(move || {
            cmd.hide::<Self>();
        });
    }
}
