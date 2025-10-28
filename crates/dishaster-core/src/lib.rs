//! Dishaster core logic and data structures

mod command_handle;
pub mod components;
pub(crate) mod constants;
pub mod resources;
pub mod rng;
pub mod sim;
mod snapshot;
pub mod systems;
pub mod trial;

/// Re-export of dishaster_models
pub mod models {
    pub use dishaster_models::*;
}

mod prelude {
    pub use derive_more::derive::{Deref, DerefMut};
    pub use dishrupt_core::{model_registry::*, prelude::*};
    pub use rand::prelude::*;

    pub use super::rng::*;
    pub use crate::Tick;
}

/// Type alias for simulation tick count
pub type Tick = u32;
