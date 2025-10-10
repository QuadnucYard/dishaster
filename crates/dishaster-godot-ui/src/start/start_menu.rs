use crate::{prelude::*, req::GameRequest};

#[derive(UITree)]
#[ui_tree]
pub struct StartMenuUI {
    #[child("%Start")]
    start_btn: ButtonA,
    #[child("%Quit")]
    quit_btn: ButtonA,
}

#[ui_tree_api]
impl UITree for StartMenuUI {}

impl Gui for StartMenuUI {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.start_btn.on_click.connect(move || {
            cmd.push_req(GameRequest::EnterLevel);
        });

        let cmd = commands.clone();
        self.quit_btn.on_click.connect(move || {
            cmd.push_req(GameRequest::Quit);
        });
    }
}
