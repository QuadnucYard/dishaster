//! Data loading and management for Dishaster simulation

mod trial_rank;
mod trial_speech;

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use dishaster_models::{GameModelRegistry, TrialCorpus};
use dishrupt_core::model_registry::*;
use serde::Deserialize;
use thiserror::Error;

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
}

/// Data loader for game assets from RON files
pub struct DataLoader {
    assets_path: PathBuf,
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
        Ok(Self { assets_path })
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
            })
        } else {
            Ok(Self { assets_path })
        }
    }

    /// Load all game data and populate the model registry
    pub fn load_all_data(&self) -> anyhow::Result<GameModelRegistry> {
        let mut registry = GameModelRegistry::default();

        // Load each registry type from separate files
        self.load_to_registry(&mut registry.levels, "levels.ron")?;
        self.load_to_registry(&mut registry.canteens, "canteens.ron")?;
        self.load_to_registry(&mut registry.dishes, "dishes.ron")?;
        self.load_to_registry(&mut registry.window_services, "window_services.ron")?;
        self.load_to_registry(&mut registry.tables, "tables.ron")?;
        self.load_to_registry(&mut registry.dispensers, "dispensers.ron")?;
        self.load_to_registry(&mut registry.collectors, "collectors.ron")?;

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
            aq_ranks: trial_rank::parse_aq_ranks(&self.load_string("trial/ranks_aq.txt")?)?,
        };

        Ok(registry)
    }

    fn load_to_registry<T>(
        &self,
        registry: &mut ModelRegistry<T>,
        filename: &str,
    ) -> anyhow::Result<()>
    where
        T: serde::de::DeserializeOwned + HasId,
    {
        let path = self.assets_path.join(filename);
        let models: Vec<T> = self
            .load_ron_file(&path)
            .with_context(|| format!("Loading {filename}"))?;
        for model in models {
            registry.intern(model.id().clone(), model);
        }
        Ok(())
    }

    fn load_ron_file<T>(&self, path: &Path) -> Result<T, DataError>
    where
        T: serde::de::DeserializeOwned,
    {
        use ron::extensions::Extensions;

        let content = std::fs::read_to_string(path)?;
        let options = ron::Options::default()
            .with_default_extension(Extensions::UNWRAP_NEWTYPES | Extensions::IMPLICIT_SOME);
        let data = options.from_str(&content)?;
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
        let content = std::fs::read_to_string(&path)?;
        let data = toml::from_str::<Items<T>>(&content)
            .with_context(|| format!("Loading corpus {path:?}"))?;
        Ok(data.item)
    }

    fn load_string(&self, filename: &str) -> anyhow::Result<String> {
        let path: PathBuf = self.assets_path.join(filename);
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Loading text file {path:?}"))?;
        Ok(content)
    }
}
