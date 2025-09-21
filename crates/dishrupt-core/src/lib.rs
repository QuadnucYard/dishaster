use std::num::NonZeroU32;

pub mod asset;
pub mod display;
pub mod prelude;
pub mod utils;

/// Reference to an entity in the core ECS world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub NonZeroU32);
