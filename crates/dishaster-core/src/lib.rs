//! Dishaster core logic and data structures

mod command_handle;
pub mod commands;
pub mod components;
pub(crate) mod constants;
pub mod resources;
pub mod sim;
pub mod snapshots;
pub mod systems;

/// Re-export of dishaster_models
pub mod models {
    pub use dishaster_models::*;
}

mod prelude {
    pub use derive_more::derive::{Deref, DerefMut};
    pub use dishrupt_core::{model_registry::*, prelude::*};
    pub use rand::prelude::*;

    pub use crate::Tick;
}

/// Type alias for simulation tick count
pub type Tick = u32;
