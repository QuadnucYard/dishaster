use std::sync::Arc;

use anyhow::{Context, Result, bail};
use dishaster_models::{GameModelRegistry, LevelConfig, ModelId};
use dishrupt_persistence::PersistentStorage;

use crate::progress::*;

/// Low-level persistence service for user progress.
struct PersistenceService<Store: PersistentStorage> {
    store: Store,
}

impl<Store: PersistentStorage> PersistenceService<Store> {
    const SAVE_FILE: &str = "save_default.ron";

    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn load_progress(&mut self, seed: u64) -> Result<UserProgress> {
        self.store
            .load_or_create(Self::SAVE_FILE, || UserProgress::new(seed))
    }

    pub fn save_progress(&mut self, progress: &UserProgress) -> Result<()> {
        self.store.save(Self::SAVE_FILE, progress)
    }
}

/// High-level facade that ties the persistence layer to model data.
pub struct ProgressService<Store: PersistentStorage> {
    inner: PersistenceService<Store>,

    registry: Arc<GameModelRegistry>,
    base_level_id: ModelId,
    progress: UserProgress,
}

impl<Store: PersistentStorage> ProgressService<Store> {
    /// Load (or create) progress and prepare a service ready to dispense levels.
    pub fn load_or_create(
        store: Store,
        registry: Arc<GameModelRegistry>,
        default_level_id: Option<ModelId>,
        seed: u64,
    ) -> Result<Self> {
        let mut inner = PersistenceService::new(store);

        let progress = inner.load_progress(seed)?;
        let base_level_id = match default_level_id {
            Some(id) => id,
            None => registry
                .levels
                .first()
                .map(|level| level.id.clone())
                .context("no level configurations available in registry")?,
        };
        if registry.levels.get_by_id(&base_level_id).is_none() {
            bail!(
                "default level {:?} is not present in the supplied registry",
                base_level_id
            );
        }
        Ok(Self {
            inner,
            registry,
            base_level_id,
            progress,
        })
    }

    /// Access the immutable progress snapshot managed by the service.
    pub fn progress(&self) -> &UserProgress {
        &self.progress
    }

    /// Produce a level configuration for the player's current day.
    pub fn level_for_current_day(&self) -> Result<LevelConfig> {
        let base = self
            .registry
            .levels
            .get_by_id(&self.base_level_id)
            .with_context(|| {
                format!(
                    "default level {:?} missing from registry at runtime",
                    self.base_level_id
                )
            })?;
        let mut level = base.clone();
        level.day = self.progress.player.current_day;
        level.seed = seed_for_day(
            self.progress.player.rng_seed,
            self.progress.player.current_day,
        );
        Ok(level)
    }

    /// Persist the outcome of the day.
    pub fn complete_day(&mut self) -> Result<()> {
        self.progress.player.current_day = self.progress.player.current_day.saturating_add(1);
        self.progress.player.rng_seed = advance_seed(self.progress.player.rng_seed);
        self.progress.meta.updated_at_utc = now_unix();
        self.inner.save_progress(&self.progress)?;
        Ok(())
    }
}

fn seed_for_day(base_seed: u64, day: u32) -> u64 {
    let day_mix = (day as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    base_seed ^ day_mix.rotate_left(13)
}

fn advance_seed(seed: u64) -> u64 {
    seed.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
