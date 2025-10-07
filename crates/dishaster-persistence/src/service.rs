use std::{fs, io::ErrorKind, path::PathBuf, sync::Arc};

use anyhow::{Context, Result, bail};
use dishaster_models::{GameModelRegistry, LevelConfig, ModelId};
use ron::ser::PrettyConfig;

use crate::data::*;

/// Handles low-level persistence concerns such as reading and writing save files.
#[derive(Debug, Clone)]
pub struct PersistenceService {
    dir: PathBuf,
    file: String,
}

impl PersistenceService {
    /// Create a new persistence service targeting the provided directory.
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            file: "save_default.ron".into(),
        }
    }

    /// Load an existing progress file or initialize a fresh record if none exists.
    pub fn load_or_create(&self, seed: u64) -> Result<UserProgress> {
        fs::create_dir_all(&self.dir)
            .with_context(|| format!("failed to create persistence dir {:?}", self.dir))?;
        let path = self.dir.join(&self.file);
        if !path.exists() {
            let progress = UserProgress::new(seed);
            self.save(&progress)?;
            return Ok(progress);
        }
        let txt = fs::read_to_string(&path)
            .with_context(|| format!("failed to read save file {:?}", path))?;
        let mut progress: UserProgress = ron::from_str(&txt).context("parse user progress RON")?;
        if progress.meta.version != USER_PROGRESS_VERSION {
            self.migrate(&mut progress)?;
        }
        Ok(progress)
    }

    /// Persist the supplied progress snapshot to disk.
    pub fn save(&self, progress: &UserProgress) -> Result<()> {
        fs::create_dir_all(&self.dir)
            .with_context(|| format!("failed to create persistence dir {:?}", self.dir))?;
        let path = self.dir.join(&self.file);
        let tmp = path.with_extension("tmp");
        let ron = ron::ser::to_string_pretty(progress, PrettyConfig::default())?;
        fs::write(&tmp, ron).with_context(|| format!("failed to write temp save {:?}", tmp))?;
        match fs::rename(&tmp, &path) {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == ErrorKind::PermissionDenied => {
                if path.exists() {
                    fs::remove_file(&path).with_context(|| {
                        format!("failed to remove old save before replacing {:?}", path)
                    })?;
                }
                fs::rename(&tmp, &path)
                    .with_context(|| format!("failed to replace save file {:?}", path))?;
                Ok(())
            }
            Err(err) => Err(err)
                .with_context(|| format!("failed to move temp save {:?} -> {:?}", tmp, path)),
        }
    }

    fn migrate(&self, _progress: &mut UserProgress) -> Result<()> {
        // Placeholder for future schema upgrades.
        Ok(())
    }
}

/// High-level facade that ties the persistence layer to model data.
pub struct ProgressService {
    store: PersistenceService,
    registry: Arc<GameModelRegistry>,
    base_level_id: ModelId,
    progress: UserProgress,
}

impl ProgressService {
    /// Load (or create) progress and prepare a service ready to dispense levels.
    pub fn load_or_create(
        dir: PathBuf,
        registry: Arc<GameModelRegistry>,
        default_level_id: Option<ModelId>,
        seed: u64,
    ) -> Result<Self> {
        let store = PersistenceService::new(dir);
        let progress = store.load_or_create(seed)?;
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
            store,
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
        self.store.save(&self.progress)?;
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use dishaster_models::{
        DinerAttributeRanges, DinerBehaviorRanges, DinerProviderModel, DinerSpawnerModel, MinMax,
        MovementRanges,
    };
    use dishrupt_core::asset::PrefabReference;
    use tempfile::tempdir;

    use super::*;

    fn sample_registry() -> GameModelRegistry {
        let mut registry = GameModelRegistry::default();
        let level = LevelConfig {
            id: ModelId::new("level_default"),
            day: 1,
            canteen: ModelId::new("canteen_default"),
            window_configurations: vec![],
            table_placements: vec![],
            tray_dispenser_placements: vec![],
            chopstick_dispenser_placements: vec![],
            collector_placements: vec![],
            diner_provider: sample_diner_provider(),
            diner_spawner: DinerSpawnerModel {
                run_length: 120.0,
                base_rate_per_min: 12.0,
                spawn_curve: vec![],
            },
            seed: 42,
        };
        registry.levels.intern(level.id.clone(), level);
        registry
    }

    fn sample_diner_provider() -> DinerProviderModel {
        DinerProviderModel {
            attributes: DinerAttributeRanges {
                hunger: MinMax::new(0.2, 0.8),
                patience: MinMax::new(10.0, 30.0),
                economic_capacity: MinMax::new(10.0, 30.0),
                price_sensitivity: MinMax::new(0.5, 1.5),
            },
            behavior: DinerBehaviorRanges {
                decisiveness: MinMax::new(0.2, 0.8),
                adaptiveness: MinMax::new(0.2, 0.8),
                leave_probability: MinMax::new(0.0, 0.2),
                observation_time: MinMax::new(1.0, 3.0),
                decision_time: MinMax::new(3.0, 6.0),
                eating_time: MinMax::new(6.0, 12.0),
            },
            movement: MovementRanges {
                movement_speed: MinMax::new(0.8, 1.2),
                avoidance_speed: MinMax::new(0.9, 1.3),
                arrival_threshold: MinMax::new(0.1, 0.3),
            },
            display_res: vec![PrefabReference::new("diner/basic")],
        }
    }

    #[test]
    fn new_user_receives_first_day_level() -> Result<()> {
        let registry = Arc::new(sample_registry());
        let dir = tempdir()?;
        let service = ProgressService::load_or_create(
            dir.path().to_path_buf(),
            Arc::clone(&registry),
            None,
            12345,
        )?;
        let level = service.level_for_current_day()?;
        assert_eq!(level.day, 1);
        assert_eq!(level.seed, super::seed_for_day(12345, 1));
        Ok(())
    }

    #[test]
    fn completing_day_advances_and_persists_progress() -> Result<()> {
        let registry = Arc::new(sample_registry());
        let dir = tempdir()?;
        let mut service = ProgressService::load_or_create(
            dir.path().to_path_buf(),
            Arc::clone(&registry),
            None,
            999,
        )?;
        let _initial = service.level_for_current_day()?;
        service.complete_day()?;
        let next_level = service.level_for_current_day()?;
        assert_eq!(next_level.day, 2);
        assert_eq!(
            next_level.seed,
            super::seed_for_day(super::advance_seed(999), 2)
        );
        drop(service);
        let service = ProgressService::load_or_create(dir.path().to_path_buf(), registry, None, 0)?;
        let level = service.level_for_current_day()?;
        assert_eq!(level.day, 2);
        assert_eq!(level.seed, super::seed_for_day(super::advance_seed(999), 2));
        Ok(())
    }
}
