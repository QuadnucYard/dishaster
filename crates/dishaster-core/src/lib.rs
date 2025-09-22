//! Dishaster core logic and data structures

pub mod components;
pub mod models;
pub mod resources;
pub mod sim;
pub mod systems;
pub mod utils;

mod prelude {
    pub use dishrupt_core::{model_registry::*, prelude::*};
    pub use rand::prelude::*;
}
