pub mod asset;
pub mod display;
mod ext;
pub mod model_registry;
pub mod prelude;
pub mod utils;

use std::num::NonZeroU64;

use bevy_ecs::entity::Entity;

/// Reference to an entity in the core ECS world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(NonZeroU64);

impl EntityId {
    /// Get the underlying raw integer ID
    pub fn to_bits(self) -> u64 {
        self.0.get()
    }
}

impl From<Entity> for EntityId {
    fn from(entity: Entity) -> Self {
        EntityId(NonZeroU64::new(entity.to_bits()).expect("Entity should never be zero"))
    }
}
