//! Dishaster core logic and data structures

#![feature(vec_deque_pop_if)]

mod command_handle;
pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod resources;
pub mod rng;
pub mod sim;
mod snapshot;
pub(crate) mod systems;
pub(crate) mod trial;

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
