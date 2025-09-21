use dishrupt_godot_ui::GuiRequest;

pub struct QuitRequest {}

impl GuiRequest for QuitRequest {}

pub struct EnterLevelRequest {}

impl GuiRequest for EnterLevelRequest {}

pub struct ExitLevelRequest {}

impl GuiRequest for ExitLevelRequest {}
