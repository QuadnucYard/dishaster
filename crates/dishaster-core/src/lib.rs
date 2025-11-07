//! Dishaster core logic and data structures

#![feature(vec_deque_pop_if)]

mod adapter;
mod command_handle;
pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod messages;
pub(crate) mod resources;
pub mod sim;
mod snapshot;
pub(crate) mod systems;
pub(crate) mod trial;

/// Re-export of dishaster_models
pub mod models {
    pub use dishaster_models::*;
}

/// Re-export of dishaster_views
pub mod views {
    pub use dishaster_views::*;
}

mod prelude {
    pub use dishrupt_core::prelude::*;
    pub use dishrupt_ecs::prelude::*;
    pub use dishrupt_rng::prelude::*;
    pub use ordered_float::NotNan;
    pub use rustc_hash::{FxHashMap, FxHashSet};

    pub use crate::{Tick, adapter::*};
}

/// Type alias for simulation tick count
pub type Tick = u32;
