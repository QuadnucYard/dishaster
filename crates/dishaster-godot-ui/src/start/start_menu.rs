use crate::prelude::*;

#[derive(UITree)]
#[ui_tree]
pub struct StartMenuGui {
    #[child("%Start")]
    start_btn: ButtonA,
    #[child("%Quit")]
    quit_btn: ButtonA,
}

#[ui_tree_api]
impl UITree for StartMenuGui {}

impl Gui for StartMenuGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.start_btn.on_click.connect(move || {
            cmd.push_req(AppRequest::EnterLevel);
        });

        let cmd = commands.clone();
        self.quit_btn.on_click.connect(move || {
            cmd.push_req(AppRequest::Quit);
        });
    }
}
