//! Backend abstraction and implementations for loading resources.

mod fs;
#[cfg(feature = "godot")]
mod godot;

use thiserror::Error;

pub use self::fs::FsBackend;
#[cfg(feature = "godot")]
pub use self::godot::GodotResourceBackend;
use crate::ResourceLocator;

/// Load errors returned by [`DataBackend`] methods.
#[derive(Debug, Error)]
pub enum LoadError {
    /// Resource not found.
    #[error("Resource not found: {0}")]
    NotFound(ResourceLocator),

    /// Locator type is unsupported.
    #[error("Unsupported location: {0}")]
    UnsupportedLocation(ResourceLocator),

    /// Underlying IO error from the backend.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other errors (transparent wrapper for [`anyhow::Error`]).
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Abstraction for reading resources from a storage backend.
pub trait DataBackend {
    /// Check if a resource exists without reading its contents.
    fn exists(&self, locator: &ResourceLocator) -> Result<bool, LoadError>;

    /// Check whether `locator` represents a file in the backend.
    fn is_file(&self, locator: &ResourceLocator) -> Result<bool, LoadError>;

    /// Check whether `locator` represents a directory in the backend.
    fn is_dir(&self, locator: &ResourceLocator) -> Result<bool, LoadError>;

    /// List the contents of a directory locator.
    fn list_dir(&self, locator: &ResourceLocator) -> Result<Vec<ResourceLocator>, LoadError>;

    /// Read raw bytes for `locator`. Returns `LoadError::NotFound` if absent.
    fn read_bytes(&self, locator: &ResourceLocator) -> Result<Vec<u8>, LoadError>;

    /// Read `String` data for `locator` converting bytes as UTF-8.
    fn read_string(&self, locator: &ResourceLocator) -> Result<String, LoadError> {
        let bytes = self.read_bytes(locator)?;
        String::from_utf8(bytes).map_err(|e| LoadError::Other(anyhow::Error::new(e)))
    }
}
