//! Data loading and management for Dishaster simulation

#[cfg(feature = "codespan")]
mod codespan;
mod trial_rank;
mod trial_speech;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, bail};
use dishaster_models::{GameModelRegistry, TrialCorpus};
use dishaster_opening_models::{CreditsData, OpeningConfig};
use dishrupt_core::model_registry::*;
use serde::Deserialize;
use thiserror::Error;

// === Data Structures ===

/// Complete set of game data assets
pub struct GameDataAssets {
    /// Game model definitions used in the simulation
    pub models: Arc<GameModelRegistry>,
    /// Opening animation configuration
    pub opening_config: OpeningConfig,
    /// Credits data
    pub credits: CreditsData,
}

// === Data Loader ===

/// Error types for data loading operations
#[derive(Error, Debug)]
pub enum DataError {
    /// File system I/O operation failed
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    /// RON parsing or deserialization failed
    #[error("Failed to parse RON data: {0}")]
    RonError(#[from] ron::error::SpannedError),
    /// TOML parsing or deserialization failed
    #[error("Failed to parse TOML data: {0}")]
    TomlError(#[from] toml::de::Error),
    /// Data validation or consistency check failed
    #[error("Data validation failed: {0}")]
    ValidationError(String),
    /// Other unspecified error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Data loader for game assets from RON files
pub struct DataLoader {
    assets_path: PathBuf,

    index: HashMap<String, String>,
}

impl DataLoader {
    /// Create a new data loader with the specified assets directory
    pub fn new(assets_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let assets_path = assets_path.as_ref().to_path_buf();
        if !std::fs::exists(&assets_path).is_ok_and(|exists| exists) {
            bail!(DataError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Assets path does not exist: {}. The current working directory is: {:?}",
                    assets_path.display(),
                    std::env::current_dir()
                ),
            )));
        }
        Ok(Self {
            assets_path,
            index: Default::default(),
        })
    }

    /// Create a new data loader, falling back to an alternative path if the primary does not exist
    pub fn new_with_fallback(
        assets_path: impl AsRef<Path>,
        fallback_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let assets_path = assets_path.as_ref().to_path_buf();
        if !std::fs::exists(&assets_path).is_ok_and(|exists| exists) {
            let fallback_path = fallback_path.as_ref().to_path_buf();
            if !std::fs::exists(&fallback_path).is_ok_and(|exists| exists) {
                bail!(DataError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "Both assets path and fallback path do not exist: {}, {}. The current working directory is: {:?}",
                        assets_path.display(),
                        fallback_path.display(),
                        std::env::current_dir()
                    ),
                )));
            }
            Ok(Self {
                assets_path: fallback_path,
                index: Default::default(),
            })
        } else {
            Ok(Self {
                assets_path,
                index: Default::default(),
            })
        }
    }

    /// Load all game data and populate the model registry
    pub fn load_all_data(&mut self) -> anyhow::Result<GameDataAssets> {
        self.index =
            load_toml(&self.assets_path.join("index.toml")).context("loading data index file")?;

        let mut registry = GameModelRegistry::default();

        // Load each registry type from separate files
        self.load_to_registry(&mut registry.levels, "levels")?;
        self.load_to_registry(&mut registry.canteens, "canteens")?;
        self.load_to_registry(&mut registry.dishes, "dishes")?;
        self.load_to_registry(&mut registry.window_services, "window_services")?;
        self.load_to_registry(&mut registry.tables, "tables")?;
        self.load_to_registry(&mut registry.dispensers, "dispensers")?;
        self.load_to_registry(&mut registry.collectors, "collectors")?;
        self.load_to_registry(&mut registry.mgmt_decisions, "mgmt_decisions")?;
        self.load_to_registry(&mut registry.mgmt_incidents, "mgmt_incidents")?;

        registry.trial = TrialCorpus {
            diner_speeches: {
                let mut diner_speeches = self.load_corpus("trial/corpus.toml")?;
                trial_speech::populate_trial_speech_items(&mut diner_speeches)?;
                diner_speeches
            },
            responses: {
                let mut responses = self.load_corpus("trial/corpus_r.toml")?;
                trial_speech::populate_trial_response_items(&mut responses)?;
                responses
            },
            qa_ranks: trial_rank::parse_qa_ranks(&self.load_string("trial/ranks_qa.txt")?)?,
            aq_ranks: trial_rank::parse_aq_ranks(&self.load_string("trial/ranks_aq.txt")?, "AQ")?,
            qq_ranks: trial_rank::parse_aq_ranks(&self.load_string("trial/ranks_qq.txt")?, "QQ")?,
            rr_ranks: trial_rank::parse_aq_ranks(&self.load_string("trial/ranks_rr.txt")?, "RR")?,
        };

        let opening_config = self
            .load_ron_file(&self.assets_path.join("opening.ron"))
            .with_context(|| "Loading opening configuration")?;

        let credits: CreditsData = self
            .load_ron_file(&self.assets_path.join("misc/credits.ron"))
            .with_context(|| "Loading credits data")?;

        Ok(GameDataAssets {
            models: registry.into(),
            opening_config,
            credits,
        })
    }

    fn load_to_registry<T>(&self, registry: &mut ModelRegistry<T>, key: &str) -> anyhow::Result<()>
    where
        T: serde::de::DeserializeOwned + HasId,
    {
        let path = self.assets_path.join(self.index.get(key).ok_or_else(|| {
            DataError::ValidationError(format!("No index entry for data file key: {}", key))
        })?);

        if path.is_file() {
            return self.load_ron_to_registry(registry, &path);
        }
        if path.is_dir() {
            // Load all RON files in the directory
            let mut has_loaded = false;
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                let entry_path = entry.path();
                if !entry_path.is_file() {
                    continue;
                }
                match self.load_ron_to_registry_single(registry, &entry_path) {
                    Ok(_) => {
                        has_loaded = true;
                    }
                    Err(e) => {
                        // Log the error but continue loading other files
                        log::error!("Failed to load data file {}: {}", entry_path.display(), e);
                    }
                }
            }
            if !has_loaded {
                log::error!(
                    "No valid data files found in directory for key {}: {}",
                    key,
                    path.display()
                );
            }
        } else {
            log::error!(
                "Data path for key {} is neither a file nor a directory: {}",
                key,
                path.display()
            );
        }
        Ok(())
    }

    fn load_ron_to_registry<T>(
        &self,
        registry: &mut ModelRegistry<T>,
        path: &Path,
    ) -> anyhow::Result<()>
    where
        T: serde::de::DeserializeOwned + HasId,
    {
        let models: Vec<T> = self
            .load_ron_file(path)
            .with_context(|| format!("Loading {}", path.display()))?;

        for model in models {
            registry.intern(model.id().clone(), model);
        }

        Ok(())
    }

    fn load_ron_to_registry_single<T>(
        &self,
        registry: &mut ModelRegistry<T>,
        path: &Path,
    ) -> anyhow::Result<()>
    where
        T: serde::de::DeserializeOwned + HasId,
    {
        let model: T = self
            .load_ron_file(path)
            .with_context(|| format!("Loading {}", path.display()))?;

        registry.intern(model.id().clone(), model);

        Ok(())
    }

    fn load_ron_file<T>(&self, path: &Path) -> Result<T, DataError>
    where
        T: serde::de::DeserializeOwned,
    {
        use ron::extensions::Extensions;

        let content = std::fs::read_to_string(path)?;
        let options = ron::Options::default().with_default_extension(
            Extensions::UNWRAP_NEWTYPES
                | Extensions::IMPLICIT_SOME
                | Extensions::UNWRAP_VARIANT_NEWTYPES,
        );
        let data = match options.from_str(&content) {
            Ok(data) => data,
            Err(e) => {
                #[cfg(feature = "codespan")]
                codespan::emit_ron_error(path, &content, &e)?;

                return Err(DataError::RonError(e));
            }
        };
        Ok(data)
    }

    fn load_corpus<T>(&self, filename: &str) -> anyhow::Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        #[derive(Deserialize)]
        struct Items<T> {
            item: Vec<T>,
        }

        let path: PathBuf = self.assets_path.join(filename);
        let data = load_toml::<Items<T>>(&path)?;
        Ok(data.item)
    }

    fn load_string(&self, filename: &str) -> anyhow::Result<String> {
        let path: PathBuf = self.assets_path.join(filename);
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Loading text file {path:?}"))?;
        Ok(content)
    }
}

/// Load and parse a TOML file into the specified data structure
pub fn load_toml<T>(path: &Path) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Reading TOML file {}", path.display()))?;
    let data = toml::from_str::<T>(&content)
        .with_context(|| format!("Parsing TOML file {}", path.display()))?;
    Ok(data)
}
