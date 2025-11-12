//! Model definitions for Dishaster save data

mod cosmetic;
mod diner_pool;
mod level;
mod perma_effects;
mod player;

pub use cosmetic::*;
pub use diner_pool::*;
pub use level::*;
pub use perma_effects::*;
pub use player::*;

mod prelude {
    pub use dishrupt_core::{
        model_registry::{HasId, ModelId},
        prelude::*,
    };
    pub use rustc_hash::{FxHashMap, FxHashSet};
    pub use serde::{Deserialize, Serialize};
}

/// Seed wrapper for RNG seeds
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Seed(u64);

impl Seed {
    /// Create a new Seed from a u64 value
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Get the underlying seed value
    pub fn get(&self) -> u64 {
        self.0
    }
}

/// Persistent simulation profile data to be saved between runs
#[derive(Debug, Default)]
pub struct SimProfile {
    /// Day index (1-based). Increment on new day/run.
    pub current_day: Day,

    /// Seed for current run's RNG
    pub rng_seed: Seed,

    /// Player-configured window setups
    pub window_configurations: Vec<WindowConfiguration>,

    /// Physical placements of canteen objects
    pub placement: CanteenPlacements,

    /// Persistent diner pool
    pub diner_profiles: Vec<DinerProfile>,

    /// Permanent management decision effects
    pub permanent_effects: PermanentEffects,
}
