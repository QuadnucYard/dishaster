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
pub struct ComponentWrapper<T>(T);

impl<T> From<T> for ComponentWrapper<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
