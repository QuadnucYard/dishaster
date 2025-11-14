//! Persistence layer for Dishaster.

mod service;

use anyhow::{Context, Result};
use dishaster_save_models::{PlayerProfile, Preferences};

pub use self::service::UserDataService;

/// Re-exports of persistence-related types.
mod reexport {
    pub use dishrupt_persistence::{Persistable, PersistentStorage, Persister};
}

pub use reexport::*;

struct PlayerProfilePersister;

impl Persister<PlayerProfile> for PlayerProfilePersister {
    fn load_bytes(data: Vec<u8>) -> Result<PlayerProfile> {
        ron::de::from_bytes(&data).context("fail to parse user progress RON")
    }

    fn dump_bytes(value: &PlayerProfile) -> Result<Vec<u8>> {
        let ron_str = ron::ser::to_string_pretty(value, Default::default())?;
        Ok(ron_str.into_bytes())
    }
}

/// Persister for preferences
pub struct PreferencesPersister;

impl Persister<Preferences> for PreferencesPersister {
    fn load_bytes(data: Vec<u8>) -> Result<Preferences> {
        toml::from_slice(&data).context("failed to deserialize preferences TOML")
    }

    fn dump_bytes(value: &Preferences) -> Result<Vec<u8>> {
        let toml_str =
            toml::to_string_pretty(value).context("failed to serialize preferences to TOML")?;
        Ok(toml_str.into_bytes())
    }
}
