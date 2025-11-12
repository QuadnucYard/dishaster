//! Utility extensions and traits for Godot integration.

mod bind;
mod call;
mod ext;

pub use bind::{BindGodot, FromGodot, IntoGodot, IntoSim};
pub use call::*;
pub use ext::{NodeExt, ObjectExt};
