//! Model definitions for Dishaster save data

mod cosmetic;
mod diner_pool;
mod level;
mod player;

pub use cosmetic::*;
pub use diner_pool::*;
pub use level::*;
pub use player::*;

mod prelude {
    pub use dishrupt_core::{
        model_registry::{HasId, ModelId},
        prelude::*,
    };
    pub use rustc_hash::{FxHashMap, FxHashSet};
    pub use serde::{Deserialize, Serialize};
}

/// Persistent simulation profile data to be saved between runs
#[derive(Debug, Default)]
pub struct SimProfile {
    /// Player-configured window setups
    pub window_configurations: Vec<WindowConfiguration>,

    /// Physical placements of canteen objects
    pub placement: CanteenPlacements,

    /// Persistent diner pool
    pub diner_profiles: Vec<DinerProfile>,
}
