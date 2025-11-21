pub(crate) mod effect;
pub mod game_main;
pub(crate) mod panic;
pub(crate) mod panic_overlay;
pub(crate) mod scenes;

mod prelude {
    pub use dishaster_data::GameDataAssets;
    pub use dishaster_persistence::UserDataService;
    pub use dishrupt_asset::AssetCatalog;
    pub use dishrupt_godot_audio::AudioManager;
    pub use dishrupt_godot_input::event::GodotInputEvent;
    pub use dishrupt_godot_scene::{SceneContext, SceneManager, SceneResources};
    pub use dishrupt_godot_ui::{GuiCommands, GuiRegistry, UITree};
    pub use dishrupt_godot_utils::{BindGodot, NodeExt};
    pub use godot::{
        classes::Node,
        global::{godot_error, godot_print},
        prelude::*,
    };
}
