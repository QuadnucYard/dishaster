//! Game entity components for state management

mod agent;
mod canteen;
mod diner;
mod physics;

pub use agent::*;
pub use canteen::*;
pub use diner::*;
pub use physics::*;

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
