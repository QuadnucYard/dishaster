//! Dishaster core logic and data structures

mod adapter;
mod command_handle;
pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod events;
pub(crate) mod messages;
pub(crate) mod resources;
pub mod sim;
mod snapshot;
pub(crate) mod systems;
pub(crate) mod utils;

/// Re-export of dishaster_models
pub mod models {
    pub use dishaster_models::*;
}

/// Re-export of dishaster_views
pub mod views {
    pub use dishaster_views::*;
}

/// Re-export of dishaster_interface
pub mod interface {
    pub use dishaster_interface::*;
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

// for testing
pub use systems::convert;
