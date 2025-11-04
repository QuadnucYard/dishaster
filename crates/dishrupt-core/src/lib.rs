pub mod asset;
pub mod display;
mod ext;
pub mod model_registry;
pub mod prelude;
pub mod ui;
pub mod utils;

use std::num::NonZeroU64;

pub use model_registry::ModelId;

/// Type alias for simulation tick count
pub type Tick = u32;

/// Reference to an entity in the core ECS world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(NonZeroU64);

impl EntityId {
    /// Create a new EntityId from a raw integer ID
    pub fn new(id: u64) -> Option<Self> {
        NonZeroU64::new(id).map(EntityId)
    }

    /// Get the underlying raw integer ID
    pub fn to_bits(self) -> u64 {
        self.0.get()
    }
}
