//! Generic persistence layer abstraction.
//!
//! Provides `PersistentStorage` trait for backend-agnostic save/load operations.
//! Includes implementations for:
//! - Filesystem storage (feature: `fs`)
//! - Godot user:// storage (feature: `godot`)

#[cfg(feature = "fs")]
mod fs;
#[cfg(feature = "godot")]
mod godot;

use anyhow::Result;
#[cfg(feature = "fs")]
pub use fs::FsStorage;
#[cfg(feature = "godot")]
pub use godot::GodotUserStorage;

/// Backend-agnostic storage abstraction for persistent data.
///
/// Implementations handle platform-specific file I/O with error recovery.
pub trait PersistentStorage: Send + Sync + 'static {
    /// Load raw bytes from storage path. Returns `None` if file doesn't exist.
    fn read(&self, path: &str) -> Result<Option<Vec<u8>>>;

    /// Write raw bytes to storage path atomically.
    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<()>;

    /// Delete file at storage path. Returns `Ok(())` even if file doesn't exist.
    fn delete(&self, path: &str) -> Result<()>;
}
