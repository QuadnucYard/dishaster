use std::num::NonZeroU64;

pub mod asset;
pub mod display;
mod ext;
pub mod model_registry;
pub mod prelude;
pub mod utils;

/// Reference to an entity in the core ECS world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub NonZeroU64);
