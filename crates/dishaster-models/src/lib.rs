//! Game model definitions and data structures

mod canteen;
mod cosmetic;
mod credit;
mod decision;
mod diner;
mod diner_pool;
mod diner_rand;
mod dish;
mod level;
mod misc;
mod trial;

pub use canteen::*;
pub use cosmetic::*;
pub use credit::*;
pub use decision::*;
pub use diner::*;
pub use diner_pool::*;
pub use diner_rand::*;
pub use dish::*;
pub use level::*;
pub use misc::*;
pub use prelude::ModelId;
pub use trial::*;

mod prelude {
    pub use dishrupt_core::{
        model_registry::{HasId, ModelId},
        prelude::*,
    };
    pub use rustc_hash::{FxHashMap, FxHashSet};
    pub use serde::{Deserialize, Serialize};

    pub use super::{Meters, MinMax, Seconds, Size, XRange};
}

/// Physical distance measurement in meters
///
/// Used for all spatial calculations including object sizes,
/// positions, and movement distances within the simulation.
pub type Meters = f32;

/// Time duration measurement in seconds
///
/// Used for timing calculations, delays, and duration-based
/// game mechanics throughout the simulation.
pub type Seconds = f32;

use dishrupt_core::model_registry::ModelRegistry;

/// Centralized registry for all game object model definitions
///
/// Manages the static configuration templates that define the properties
/// and behaviors of all entities in the simulation. Uses type-safe handles
/// to reference models efficiently without duplicating data.
#[derive(Default)]
pub struct GameModelRegistry {
    /// Level configurations defining initial setups
    pub levels: ModelRegistry<LevelConfig>,
    /// Canteen layout and structural configurations
    pub canteens: ModelRegistry<CanteenModel>,
    /// Food service window configurations and constraints
    pub window_services: ModelRegistry<WindowServiceModel>,
    /// Dish definitions with pricing and characteristics
    pub dishes: ModelRegistry<DishModel>,
    /// Table models with seating and comfort properties
    pub tables: ModelRegistry<TableModel>,
    /// Tray and utensil dispenser configurations
    pub dispensers: ModelRegistry<DispenserModel>,
    /// Dish collection point configurations
    pub collectors: ModelRegistry<CollectorModel>,

    /// Trial corpus
    pub trial: TrialCorpus,
}
