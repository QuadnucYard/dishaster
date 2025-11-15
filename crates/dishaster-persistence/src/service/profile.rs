//! Player profile persistence service with write-through caching.

use std::{
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use dishaster_save_models::{PlayerProfile, ProfileMeta, USER_PROGRESS_VERSION};
use dishrupt_core::prelude::*;
use dishrupt_persistence::PersistentStorage;

use crate::{PersistenceFormat, RonFormat};

/// Service for managing player profile data with caching.
///
/// Profiles are stored as RON for complex nested structures. The service maintains
/// an in-memory cache and automatically updates timestamps on mutations.
pub struct ProfileService {
    storage: Arc<dyn PersistentStorage>,
    cache: RwLock<Option<PlayerProfile>>,
}

impl ProfileService {
    const FILE: &str = "player.ron";

    /// Create a new profile service backed by the given storage.
    pub fn new(storage: Arc<dyn PersistentStorage>) -> Self {
        Self {
            storage,
            cache: RwLock::new(None),
        }
    }

    /// Load from storage or create new profile. Does not acquire cache.
    fn load_from_storage_or_default(&self) -> Result<PlayerProfile> {
        if let Ok(Some(bytes)) = self.storage.read(Self::FILE) {
            let profile: PlayerProfile = RonFormat::load_bytes(bytes)?;
            return Ok(profile);
        }

        Ok(new_profile())
    }

    /// Write profile to storage atomically.
    fn persist_atomic(&self, profile: &PlayerProfile) -> Result<()> {
        let bytes = RonFormat::dump_bytes(profile)?;
        self.storage.write_atomic(Self::FILE, &bytes)
    }

    /// Update the in-memory cache with the given profile.
    fn update_cache(&self, profile: &PlayerProfile) -> Result<()> {
        if let Ok(mut cache) = self.cache.write() {
            *cache = Some(profile.clone());
        }
        Ok(())
    }

    /// Create a new player profile and persist it.
    pub fn create(&self) -> Result<()> {
        let profile = new_profile();
        self.save(&profile)
    }

    /// Load profile, using cached value if available.
    pub fn load(&self) -> Result<PlayerProfile> {
        if let Some(Some(cached)) = self.cache.read().ok().as_deref() {
            return Ok(cached.clone());
        }

        let profile = self.load_from_storage_or_default()?;
        self.update_cache(&profile)?;

        Ok(profile)
    }

    /// Save profile, updating both storage and cache.
    pub fn save(&self, profile: &PlayerProfile) -> Result<()> {
        self.persist_atomic(profile)?;
        self.update_cache(profile)?;
        Ok(())
    }

    /// Apply atomic mutation to profile.
    ///
    /// The mutator function receives mutable profile and can modify it.
    /// Changes are persisted atomically with automatic timestamp update.
    pub fn update<F>(&self, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut PlayerProfile) -> Result<()>,
    {
        let mut guard = self
            .cache
            .write()
            .map_err(|e| anyhow::anyhow!("prefs lock poisoned: {}", e))?;

        if guard.is_none() {
            *guard = Some(self.load_from_storage_or_default()?);
        }

        // SAFETY: guard is Some now
        let profile = guard.as_mut().unwrap();
        mutator(profile)?;
        profile.meta.updated_at_utc = now_unix(); // Update timestamp

        self.persist_atomic(profile)?;

        Ok(())
    }

    /// Add a hint to the seen hints set.
    pub fn update_seen_hint(&self, new_hint: EcoString) -> Result<()> {
        self.update(|profile| {
            profile.seen_hints.insert(new_hint);
            Ok(())
        })
    }
}

/// Create a new player profile with default values.
fn new_profile() -> PlayerProfile {
    let now = now_unix();
    PlayerProfile {
        meta: ProfileMeta {
            version: USER_PROGRESS_VERSION,
            created_at_utc: now,
            updated_at_utc: now,
        },
        progress: None,
        aggregates: Default::default(),
        daily_history: Vec::new(),
        layout: Default::default(),
        diner_pool: Default::default(),
        permanent_effects: Default::default(),
        seen_hints: Default::default(),
    }
}

/// Get current Unix timestamp in seconds.
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX EPOCH")
        .as_secs()
}
