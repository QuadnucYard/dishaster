use dishrupt_godot_ui::GuiRequest;

pub struct QuitRequest;

impl GuiRequest for QuitRequest {}

pub struct EnterLevelRequest;

impl GuiRequest for EnterLevelRequest {}

pub struct ExitLevelRequest;

impl GuiRequest for ExitLevelRequest {}

pub struct StartRunRequest;

impl GuiRequest for StartRunRequest {}

pub struct NextDayRequest;

impl GuiRequest for NextDayRequest {}

pub struct EndDayRequest;

impl GuiRequest for EndDayRequest {}

/// Request to change the simulation ticks-per-second rate.
pub struct SetTpsRequest(pub f32);

impl GuiRequest for SetTpsRequest {}
