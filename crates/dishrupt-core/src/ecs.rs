//! Bevy ECS utilities.

use derive_more::derive::{Deref, DerefMut};

use crate::prelude::*;

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
