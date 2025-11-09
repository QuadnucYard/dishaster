use crate::prelude::*;

#[derive(UITree)]
#[ui_tree]
pub struct SettlementGui {
    #[child("%ConfirmButton")]
    confirm_btn: ButtonA,
}

#[ui_tree_api]
impl UITree for SettlementGui {}

impl Gui for SettlementGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.confirm_btn.on_click.connect(move || {
            cmd.push_req(GameRequest::ConfirmSettlement);
        });
    }
}
