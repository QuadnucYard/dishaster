//! Preferences persistence service with write-through caching.

use std::sync::{Arc, RwLock};

use anyhow::Result;
use dishaster_save_models::Preferences;
use dishrupt_persistence::PersistentStorage;

use crate::{PersistenceFormat, TomlFormat};

/// Service for managing user preferences with caching.
///
/// Preferences are stored as TOML for human readability. The service maintains
/// an in-memory cache to avoid repeated file I/O.
pub struct PreferencesService {
    storage: Arc<dyn PersistentStorage>,
    cache: RwLock<Option<Preferences>>,
}

impl PreferencesService {
    const FILE: &str = "preferences.toml";

    /// Create a new preferences service backed by the given storage.
    pub fn new(storage: Arc<dyn PersistentStorage>) -> Self {
        Self {
            storage,
            cache: RwLock::new(None),
        }
    }

    /// Load from storage or return defaults. Does not acquire cache.
    fn load_from_storage_or_default(&self) -> Result<Preferences> {
        if let Ok(Some(bytes)) = self.storage.read(Self::FILE) {
            let prefs: Preferences = TomlFormat::load_bytes(bytes)?;
            return Ok(prefs);
        }

        // return defaults if missing
        Ok(Preferences::default())
    }

    /// Write preferences to storage atomically.
    fn persist_atomic(&self, prefs: &Preferences) -> Result<()> {
        let bytes = TomlFormat::dump_bytes(prefs)?;
        self.storage.write_atomic(Self::FILE, &bytes)
    }

    /// Update the in-memory cache with the given preferences.
    fn update_cache(&self, prefs: &Preferences) -> Result<()> {
        if let Ok(mut cache) = self.cache.write() {
            *cache = Some(prefs.clone());
        }
        Ok(())
    }

    /// Load preferences, using cached value if available.
    pub fn load(&self) -> Result<Preferences> {
        if let Some(Some(cached)) = self.cache.read().ok().as_deref() {
            return Ok(cached.clone());
        }

        let prefs = self.load_from_storage_or_default()?;
        self.update_cache(&prefs)?;

        Ok(prefs)
    }

    /// Save preferences, updating both storage and cache.
    pub fn save(&self, prefs: &Preferences) -> Result<()> {
        self.persist_atomic(prefs)?;
        self.update_cache(prefs)?;
        Ok(())
    }

    /// Apply atomic mutation to preferences.
    ///
    /// The mutator function receives mutable preferences and can modify them.
    /// Changes are persisted atomically. Returns updated preferences on success.
    pub fn update<F>(&self, mutator: F) -> Result<Preferences>
    where
        F: FnOnce(&mut Preferences) -> Result<()>,
    {
        let mut guard = self
            .cache
            .write()
            .map_err(|e| anyhow::anyhow!("prefs lock poisoned: {}", e))?;

        if guard.is_none() {
            *guard = Some(self.load_from_storage_or_default()?);
        }
        // SAFETY: guard is Some now
        let prefs = guard.as_mut().unwrap();
        mutator(prefs)?;

        self.persist_atomic(prefs)?;

        Ok(prefs.clone())
    }
}
