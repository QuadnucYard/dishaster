mod def;
mod gaming;
pub mod req;
mod start;

pub use def::register_guis;
pub use gaming::*;
pub use start::*;

mod prelude {
    pub use dishrupt_godot_ui::{elem::*, *};
    pub use signals2::*;
}
