//! Dishaster core logic and data structures

pub mod components;
pub mod constants;
pub mod models;
pub mod resources;
pub mod sim;
pub mod systems;

mod prelude {
    pub use derive_more::derive::{Deref, DerefMut};
    pub use dishrupt_core::{model_registry::*, prelude::*};
    pub use rand::prelude::*;
}
