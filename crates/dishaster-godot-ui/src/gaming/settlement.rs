use crate::{
    prelude::*,
    req::{AppRequest, GameRequest},
};

#[derive(UITree)]
#[ui_tree]
pub struct SettlementGui {
    #[child("%NextDayButton")]
    next_day: ButtonA,
    #[child("%ExitButton")]
    exit_button: ButtonA,
}

#[ui_tree_api]
impl UITree for SettlementGui {}

impl Gui for SettlementGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.next_day.on_click.connect(move || {
            cmd.push_req(GameRequest::NextDay);
        });

        let cmd = commands.clone();
        self.exit_button.on_click.connect(move || {
            cmd.push_req(AppRequest::ExitLevel);
        });
    }
}
