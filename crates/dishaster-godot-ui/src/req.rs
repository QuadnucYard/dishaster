use dishrupt_godot_ui::GuiRequest;

pub enum GameRequest {
    /// Quite the game application.
    Quit,
    /// Start a new game run at the current level.
    EnterLevel,
    /// Exit the current game run and return to the main menu.
    ExitLevel,
    /// Start the current run (from preparation phase).
    StartRun,
    /// End the current run immediately.
    EndRun,
    /// Skip to the next day (from settlement phase).
    NextDay,
    /// Change the simulation ticks-per-second rate.
    SetTps(f32),
}

impl GuiRequest for GameRequest {}
