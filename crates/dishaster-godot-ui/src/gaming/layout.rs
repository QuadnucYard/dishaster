use crate::prelude::*;

#[derive(UITree)]
#[ui_tree]
pub struct GamingLayout {}

#[ui_tree_api]
impl UITree for GamingLayout {}

impl Gui for GamingLayout {
    fn start(&mut self, _cmd: GuiCommands) {}
}
