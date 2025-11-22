//! Godot resource loader for Fluent resources.

mod loader;
mod resource;

pub use loader::{GodotResLoader, GodotResLoaderBuilder};
pub use resource::{FluentFormatLoader, FluentResource};
