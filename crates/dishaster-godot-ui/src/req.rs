use dishaster_models::PricingMethod;
use dishrupt_core::EntityId;
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
    /// Enable or disable debug mode.
    SetDebugMode(bool),
    // /// Request opening the dish price editor for a given dish entity.
    // OpenDishPriceEditor(EntityId),
    /// Apply the player's chosen pricing to a dish slot.
    ApplyDishPrice {
        dish: EntityId,
        method: PricingMethod,
    },
}

impl GuiRequest for GameRequest {}
