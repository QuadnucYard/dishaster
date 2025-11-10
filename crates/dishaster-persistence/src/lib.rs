//! Persistence layer for Dishaster.

mod service;

use anyhow::{Context, Result};
use dishaster_save_models::PlayerProfile;

pub use self::service::PlayerService;

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
