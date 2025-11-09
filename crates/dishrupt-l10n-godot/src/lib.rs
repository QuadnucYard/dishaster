//! Statically localize UI elements in Godot.
pub mod localized;
mod manager;

// Re-export for convenience
pub use dishrupt_l10n::{fluent, init, tr, try_tr_plain};
pub use manager::LocalizationManager;
