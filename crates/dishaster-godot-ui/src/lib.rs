mod def;
mod gaming;
mod start;

pub use def::register_guis;
pub use gaming::*;
pub use start::*;

mod prelude {
    pub use dishaster_ui_protocol::*;
    pub use dishrupt_godot_ui::*;
    pub use dishrupt_godot_ui_macros::*;
    pub use dishrupt_godot_widgets::*;
    pub use dishrupt_l10n_godot::tr;
    pub use signals2::*;
}
