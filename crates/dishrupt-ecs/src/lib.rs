//! Bevy ECS utilities.

pub mod display;
pub mod prelude;

use derive_more::derive::{Deref, DerefMut};
use dishrupt_core::EntityId;
use prelude::*;

/// Turn a type into a Bevy component
#[derive(Component, Default, Deref, DerefMut)]
pub struct CompWrapper<T>(T);

impl<T> From<T> for CompWrapper<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// Extension trait to convert any type into a CompWrapper
pub trait IntoComponent {
    /// Wrap this value in a CompWrapper for use as a Bevy resource
    fn into_comp(self) -> CompWrapper<Self>
    where
        Self: Sized,
    {
        CompWrapper::from(self)
    }
}

impl<T> IntoComponent for T {}

/// Turn a type into a Bevy resource
#[derive(Resource, Default, Deref, DerefMut)]
pub struct ResWrapper<T>(T);

impl<T> From<T> for ResWrapper<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// Extension trait to convert any type into a ResourceWrapper
pub trait IntoResource {
    /// Wrap this value in a ResourceWrapper for use as a Bevy resource
    fn into_res(self) -> ResWrapper<Self>
    where
        Self: Sized,
    {
        ResWrapper::from(self)
    }
}

impl<T> IntoResource for T {}

// === Entity ↔ EntityId conversions ===

/// Extension trait to convert ECS Entity to EntityId
pub trait ToEntityId {
    /// The output type
    type Output;

    /// Convert to EntityId
    fn to_entity_id(self) -> Self::Output;
}

impl ToEntityId for Entity {
    type Output = EntityId;

    fn to_entity_id(self) -> Self::Output {
        EntityId::new(self.to_bits()).expect("Entity should never be zero")
    }
}

impl ToEntityId for Option<Entity> {
    type Output = Option<EntityId>;

    fn to_entity_id(self) -> Self::Output {
        self.map(|e| e.to_entity_id())
    }
}

/// Extension trait to convert EntityId to ECS Entity
pub trait ToEntity {
    /// Convert to Entity
    fn to_entity(self) -> Entity;
}

impl ToEntity for EntityId {
    fn to_entity(self) -> Entity {
        Entity::from_bits(self.to_bits())
    }
}
