mod def;
mod gaming;
mod start;

pub use def::register_guis;
pub use gaming::*;
pub use start::*;

mod prelude {
    pub use dishaster_ui_protocol::*;
    pub use dishrupt_godot_ui::{elem::*, *};
    pub use signals2::*;
}
