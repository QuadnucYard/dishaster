//! Persistence layer for Dishaster game data.
//!
//! Provides high-level services for saving and loading:
//! - Player profiles (progress, stats, diner pool)
//! - User preferences (settings, audio levels)
//!
//! Uses different serialization formats:
//! - TOML for preferences
//! - RON for profiles

mod service;

use anyhow::{Context, Result};

pub use self::service::{PreferencesService, ProfileService, UserDataService};

/// Re-exports of persistence-related types.
mod reexport {
    pub use dishrupt_persistence::PersistentStorage;
}

pub use reexport::*;

/// Serialization format abstraction for persistence.
trait PersistenceFormat {
    /// Deserialize bytes into a typed value.
    fn load_bytes<T: serde::de::DeserializeOwned>(data: Vec<u8>) -> Result<T>
    where
        Self: Sized;

    /// Serialize a value into bytes.
    fn dump_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>>;
}

/// TOML format serializer for preferences.
pub struct TomlFormat;

impl PersistenceFormat for TomlFormat {
    fn load_bytes<T: serde::de::DeserializeOwned>(data: Vec<u8>) -> Result<T> {
        toml::from_slice(&data).context("failed to deserialize TOML data")
    }

    fn dump_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
        let toml_str = toml::to_string_pretty(value).context("failed to serialize data to TOML")?;
        Ok(toml_str.into_bytes())
    }
}

/// RON format serializer for player profiles.
pub struct RonFormat;

impl PersistenceFormat for RonFormat {
    fn load_bytes<T: serde::de::DeserializeOwned>(data: Vec<u8>) -> Result<T> {
        ron::de::from_bytes(&data).context("failed to parse RON data")
    }

    fn dump_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
        let ron_str = ron::ser::to_string(value).context("failed to serialize data to RON")?;
        Ok(ron_str.into_bytes())
    }
}
