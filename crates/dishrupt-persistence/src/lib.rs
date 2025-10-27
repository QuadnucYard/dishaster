//! General persistence layer.

#[cfg(feature = "fs")]
pub mod fs;

use anyhow::Result;
#[cfg(feature = "fs")]
pub use fs::FsStorage;

/// Abstraction over persistent storage backends.
pub trait PersistentStorage {
    /// Load existing data from the specified path, or create new data using the provided initializer.
    fn load_or_create<T: Persistable>(&mut self, path: &str, init: impl FnOnce() -> T)
    -> Result<T>;

    /// Save the provided data to the specified path.
    fn save<T: Persistable>(&mut self, path: &str, data: &T) -> Result<()>;
}

/// Trait for types that can be persisted to and loaded from bytes.
pub trait Persistable {
    /// Create an instance of the type from the provided byte slice.
    fn from_bytes(data: Vec<u8>) -> Result<Self>
    where
        Self: Sized;

    /// Create an instance of the type from the provided byte slice.
    fn from_bytes_slice(data: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        Self::from_bytes(data.to_vec())
    }

    /// Convert the instance into a byte vector for storage.
    fn to_bytes(&self) -> Result<Vec<u8>>;
}

impl Persistable for String {
    fn from_bytes(data: Vec<u8>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(String::from_utf8(data)?)
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.as_bytes().to_vec())
    }
}
