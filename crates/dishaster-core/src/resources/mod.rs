//! Simulation resources and global state management

mod time;

use std::sync::Arc;

use dishaster_navigation::{CollisionGrid, CrowdCostField};
use rand_chacha::ChaCha8Rng;
pub use time::Time;

use crate::{models::*, prelude::*};

/// Turn a type into a Bevy resource
#[derive(Resource, Default, Deref, DerefMut)]
pub struct ResourceWrapper<T>(T);

impl<T> From<T> for ResourceWrapper<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// Root entity for all display-related objects in the scene
#[derive(Resource)]
pub struct DisplayRoot(pub Entity);

/// Cryptographically secure random number generator for deterministic simulation
///
/// Uses ChaCha8 algorithm to ensure reproducible randomness across simulation runs
/// when initialized with the same seed. Essential for deterministic gameplay
/// and testing scenarios.
#[derive(Resource, Deref, DerefMut)]
pub struct GameRng(ChaCha8Rng);

impl GameRng {
    /// Create a new deterministic RNG from a 64-bit seed
    pub fn new(seed: u64) -> Self {
        let mut seed_bytes = [0u8; 32];
        seed_bytes[..8].copy_from_slice(&seed.to_le_bytes());
        Self(ChaCha8Rng::from_seed(seed_bytes))
    }
}

/// Resource wrapper for CollisionGrid
pub type CollisionGridRes = ResourceWrapper<CollisionGrid>;

/// Resource wrapper for CrowdCostField
pub type CrowdFieldRes = ResourceWrapper<CrowdCostField>;

/// Global canteen configuration and layout information
///
/// Contains the physical layout, dimensions, entrance/exit locations,
/// and window positions for the dining hall. Acts as the spatial
/// foundation for all simulation activities.
#[derive(Resource)]
pub struct Canteen {
    /// Static configuration model defining canteen layout and properties
    pub model: CanteenModel,
}

/// Diner generation template and behavioral parameter provider
///
/// Defines the statistical ranges and probability distributions used
/// to generate diverse diner personalities, attributes, and behaviors.
/// Each generated diner derives their characteristics from these templates.
#[derive(Resource)]
pub struct DinerProvider {
    /// Configuration model containing generation parameters and ranges
    pub model: DinerProviderModel,
}

/// Diner spawning controller managing timing and flow control
///
/// Handles when and how frequently new diners enter the canteen.
/// Tracks spawn timing, completion status, and coordinates with
/// the day completion logic.
#[derive(Resource)]
pub struct DinerSpawner {
    /// Configuration model defining spawn timing and flow parameters
    pub model: DinerSpawnerModel,
    /// Countdown timer until next diner spawn (in seconds)
    pub next_spawn_timer: f64,
    /// Unique ID for the next diner to be spawned
    pub next_diner_id: u32,
    /// Whether the spawning period has ended for the current day
    pub spawning_finished: bool,
}

impl DinerSpawner {
    /// Check if the spawning period has completed based on simulation time
    pub fn is_spawning_complete(&self, current_time: f64) -> bool {
        current_time >= self.model.run_length as f64
    }
}

/// Day progression and completion tracking system
///
/// Monitors the state of the current simulation day, tracking active
/// diners and determining when daily objectives are met. Coordinates
/// with spawning systems to determine overall day completion.
#[derive(Resource, Default)]
pub struct DayStatus {
    /// Number of diners currently active in the canteen
    pub current_diner_count: usize,
}

/// Resource wrapper for Arc<GameModelRegistry>
pub type GameModelRegistryRes = ResourceWrapper<Arc<GameModelRegistry>>;

/// Resource wrapper for LevelConfig
pub type LevelConfigRes = ResourceWrapper<LevelConfig>;
