use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use dishaster_models::{
    CanteenLayoutState, CanteenPlacements, GameModelRegistry, LevelConfig, ModelId, SimProfile,
};
use dishaster_save_models::{
    LevelSetupState, PlayerProfile, PlayerProgress, ProfileMeta, USER_PROGRESS_VERSION,
};
use dishrupt_core::prelude::*;
use dishrupt_persistence::PersistentStorage;

use crate::PlayerProfilePersister;

/// Low-level persistence service for user progress.
struct PersistenceService<Store: PersistentStorage> {
    store: Store,
}

impl<Store: PersistentStorage> PersistenceService<Store> {
    const SAVE_FILE: &str = "save_default.ron";

    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn load_progress(&mut self, default_level: &LevelConfig) -> Result<PlayerProfile> {
        match self
            .store
            .load_or_create_with::<_, PlayerProfilePersister>(Self::SAVE_FILE, || {
                new_profile(default_level)
            }) {
            Ok(profile) => Ok(profile),
            Err(err) => {
                log::warn!(
                    "Failed to load progress from {}: {}. Creating new profile.",
                    Self::SAVE_FILE,
                    err
                );
                let profile = new_profile(default_level);
                self.store
                    .save_with::<_, PlayerProfilePersister>(Self::SAVE_FILE, &profile)?;
                Ok(profile)
            }
        }
    }

    pub fn save_progress(&mut self, profile: &PlayerProfile) -> Result<()> {
        self.store
            .save_with::<_, PlayerProfilePersister>(Self::SAVE_FILE, profile)
    }
}

/// High-level facade that ties the persistence layer to model data.
pub struct PlayerService<Store: PersistentStorage> {
    inner: PersistenceService<Store>,

    profile: PlayerProfile,
}

impl<Store: PersistentStorage> PlayerService<Store> {
    /// Load (or create) progress and prepare a service ready to dispense levels.
    pub fn load_or_create(
        store: Store,
        registry: Arc<GameModelRegistry>,
        default_level_id: Option<ModelId>,
    ) -> Result<Self> {
        let mut inner = PersistenceService::new(store);

        let default_level = match default_level_id {
            Some(id) => registry
                .levels
                .get_by_id(&id)
                .context("level does not exist")?,
            None => registry
                .levels
                .first()
                .context("no level configurations available in registry")?,
        };
        let progress = inner.load_progress(default_level)?;
        Ok(Self {
            inner,
            profile: progress,
        })
    }

    /// Access the immutable progress snapshot managed by the service.
    pub fn profile(&self) -> &PlayerProfile {
        &self.profile
    }

    /// Produce a level configuration for the player's current day.
    pub fn level_for_current_day(&self) -> Result<LevelSetupState> {
        let level = LevelSetupState {
            level_id: self.profile.progress.level_id.clone(),
            canteen: self.profile.layout.clone(),
            day: self.profile.progress.current_day,
            seed: self.profile.progress.rng_seed,
            diner_pool: self.profile.diner_pool.profiles.clone(),
        };
        Ok(level)
    }

    /// Save simulation profile data after completing a day.
    pub fn save_profile(&mut self, profile: SimProfile) -> Result<()> {
        self.profile.meta.updated_at_utc = now_unix();

        self.profile.progress.current_day = profile.current_day;
        self.profile.progress.rng_seed = profile.rng_seed;

        self.profile.layout.window_configurations = profile.window_configurations;
        self.profile.layout.placement = profile.placement;
        self.profile.diner_pool.profiles = profile.diner_profiles;

        self.inner.save_progress(&self.profile)?;
        Ok(())
    }

    /// Update shown hints in progress
    pub fn update_seen_hint(&mut self, new_hint: EcoString) {
        self.profile.progress.seen_hints.insert(new_hint);
    }

    /// Save current progress to storage
    pub fn save(&mut self) -> Result<()> {
        self.profile.meta.updated_at_utc = now_unix();
        self.inner.save_progress(&self.profile)?;
        Ok(())
    }
}

/// Create a brand-new progress record for first-time players.
fn new_profile(default_level: &LevelConfig) -> PlayerProfile {
    let now = now_unix();
    PlayerProfile {
        meta: ProfileMeta {
            version: USER_PROGRESS_VERSION,
            created_at_utc: now,
            updated_at_utc: now,
        },
        progress: PlayerProgress {
            level_id: default_level.id.clone(),
            current_day: default_level.start_day,
            reputation: 50.0,
            rng_seed: default_level.seed,
            seen_hints: Default::default(),
        },
        aggregates: Default::default(),
        layout: CanteenLayoutState {
            window_configurations: default_level.window_configurations.clone(),
            placement: CanteenPlacements {
                tables: default_level.table_placements.clone(),
                tray_dispensers: default_level.tray_dispenser_placements.clone(),
                chopstick_dispensers: default_level.chopstick_dispenser_placements.clone(),
                collectors: default_level.collector_placements.clone(),
            },
        },
        diner_pool: Default::default(),
    }
}

pub(crate) fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX EPOCH")
        .as_secs()
}
