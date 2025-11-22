//! Data loading and management for Dishaster simulation

#[cfg(feature = "codespan")]
mod codespan;
mod trial_speech;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use dishaster_models::{GameModelRegistry, TrialCorpus};
use dishaster_opening_models::{CreditsData, EndingModel, OpeningConfig};
use dishrupt_asset::{
    AssetCatalog, AssetKind, ResolveError, ResourceLocator, backend::DataBackend,
};
use dishrupt_core::{model_registry::*, prelude::EcoString};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use thiserror::Error;

// === Data Structures ===

/// Complete set of game data assets
pub struct GameDataAssets {
    /// Game model definitions used in the simulation
    pub models: Arc<GameModelRegistry>,
    /// Ending definitions
    pub endings: FxHashMap<EcoString, EndingModel>,
    /// Opening animation configuration
    pub opening_config: OpeningConfig,
    /// Credits data
    pub credits: CreditsData,
}

// === Data Loader ===

/// Error types for data loading operations
#[derive(Error, Debug)]
pub enum DataError {
    /// Resource path could not be resolved
    #[error("Failed to resolve resource: {0}")]
    Resolve(#[from] ResolveError),
    /// File system I/O operation failed
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    /// RON parsing or deserialization failed
    #[error("Failed to parse RON data: {0}")]
    Ron(#[from] ron::error::SpannedError),
    /// TOML parsing or deserialization failed
    #[error("Failed to parse TOML data: {0}")]
    Toml(#[from] toml::de::Error),
    /// Data validation or consistency check failed
    #[error("Data validation failed: {0}")]
    Validation(String),
    /// Other unspecified error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Data loader for game assets from RON files
pub struct DataLoader {
    catalog: AssetCatalog,
    backend: Box<dyn DataBackend>,

    index: FxHashMap<String, String>,
}

impl DataLoader {
    /// Create a new data loader with the specified assets directory
    pub fn new(catalog: AssetCatalog, backend: impl DataBackend + 'static) -> Self {
        Self {
            catalog,
            backend: Box::new(backend),
            index: Default::default(),
        }
    }

    /// Create a data loader that reads from the filesystem at the given path
    pub fn from_fs(assets_path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let catalog = AssetCatalog::default();
        let backend = dishrupt_asset::backend::FsBackend::new(assets_path)?;
        Ok(Self::new(catalog, backend))
    }

    /// Load all game data and populate the model registry
    pub fn load_all_data(&mut self) -> anyhow::Result<GameDataAssets> {
        self.index = self
            .load_toml("index.toml")
            .context("loading data index file")?;

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

        // Load reputation configuration
        registry.reputation_config = self
            .load_ron_file("configs/reputation.ron")
            .context("Loading reputation configuration")?;

        // Load ordering configuration
        registry.ordering_config = self
            .load_ron_file("configs/ordering.ron")
            .context("Loading ordering configuration")?;

        // Load decision configuration
        registry.decision_config = self
            .load_ron_file("configs/decision.ron")
            .context("Loading decision configuration")?;

        // Load trial configuration
        registry.trial_config = self
            .load_ron_file("configs/trial.ron")
            .context("Loading trial configuration")?;

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
            qa_ranks: self.load_bincode("trial/ranks_qa.bin")?,
            aq_ranks: self.load_bincode("trial/ranks_aq.bin")?,
            qq_ranks: self.load_bincode("trial/ranks_qq.bin")?,
            rr_ranks: self.load_bincode("trial/ranks_rr.bin")?,
        };

        let endings = self
            .load_ron_file("endings.ron")
            .context("Loading endings data")?;

        let opening_config = self
            .load_ron_file("configs/opening.ron")
            .context("Loading opening configuration")?;

        let credits: CreditsData = self
            .load_ron_file("misc/credits.ron")
            .context("Loading credits data")?;

        Ok(GameDataAssets {
            models: registry.into(),
            endings,
            opening_config,
            credits,
        })
    }

    fn load_to_registry<T>(&self, registry: &mut ModelRegistry<T>, key: &str) -> anyhow::Result<()>
    where
        T: serde::de::DeserializeOwned + HasId,
    {
        let path = self.index.get(key).ok_or_else(|| {
            DataError::Validation(format!("No index entry for data file key: {}", key))
        })?;
        let loc = self.catalog.resolve(AssetKind::Data, path)?;
        if self.backend.is_file(&loc)? {
            return self.load_ron_to_registry(registry, &loc);
        }
        if self.backend.is_dir(&loc)? {
            // Load all RON files in the directory
            let mut has_loaded = false;
            for entry in self.backend.list_dir(&loc)? {
                if !self.backend.is_file(&entry)? {
                    continue;
                }
                match self.load_ron_to_registry_single(registry, &entry) {
                    Ok(_) => {
                        has_loaded = true;
                    }
                    Err(e) => {
                        // Log the error but continue loading other files
                        log::error!("Failed to load data file {entry}: {e}");
                    }
                }
            }
            if !has_loaded {
                log::error!("No valid data files found in directory for key {key}: {loc}");
            }
        } else {
            log::error!("Data path for key {key} is neither a file nor a directory: {loc}");
        }
        Ok(())
    }

    fn load_ron_to_registry<T>(
        &self,
        registry: &mut ModelRegistry<T>,
        loc: &ResourceLocator,
    ) -> anyhow::Result<()>
    where
        T: serde::de::DeserializeOwned + HasId,
    {
        let models: Vec<T> = self
            .load_ron_file_resolved(loc)
            .with_context(|| format!("Loading {loc}"))?;

        for model in models {
            registry.intern(model.id().clone(), model);
        }

        Ok(())
    }

    fn load_ron_to_registry_single<T>(
        &self,
        registry: &mut ModelRegistry<T>,
        loc: &ResourceLocator,
    ) -> anyhow::Result<()>
    where
        T: serde::de::DeserializeOwned + HasId,
    {
        let model: T = self
            .load_ron_file_resolved(loc)
            .with_context(|| format!("Loading {loc}"))?;

        registry.intern(model.id().clone(), model);

        Ok(())
    }

    fn load_ron_file<T>(&self, path: &str) -> Result<T, DataError>
    where
        T: serde::de::DeserializeOwned,
    {
        self.load_ron_file_resolved(&self.catalog.resolve(AssetKind::Data, path)?)
    }

    fn load_ron_file_resolved<T>(&self, loc: &ResourceLocator) -> Result<T, DataError>
    where
        T: serde::de::DeserializeOwned,
    {
        use ron::extensions::Extensions;

        let content = self.backend.read_bytes(loc).unwrap();
        let options = ron::Options::default().with_default_extension(
            Extensions::UNWRAP_NEWTYPES
                | Extensions::IMPLICIT_SOME
                | Extensions::UNWRAP_VARIANT_NEWTYPES,
        );
        let data = match options.from_bytes(&content) {
            Ok(data) => data,
            Err(e) => {
                #[cfg(feature = "codespan")]
                codespan::emit_ron_error(loc, str::from_utf8(&content).unwrap(), &e)?;

                return Err(DataError::Ron(e));
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

        let data = self.load_toml::<Items<T>>(filename)?;
        Ok(data.item)
    }

    /// Load and parse a TOML file into the specified data structure
    fn load_toml<T>(&self, path: &str) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let loc = self.catalog.resolve(AssetKind::Data, path)?;
        let bytes = self
            .backend
            .read_bytes(&loc)
            .with_context(|| format!("Reading TOML file: {loc}"))?;
        let data =
            toml::from_slice::<T>(&bytes).with_context(|| format!("Parsing TOML file: {loc}"))?;
        Ok(data)
    }

    fn load_bincode<T: bincode::Decode<()>>(&self, filename: &str) -> anyhow::Result<T> {
        let loc = self.catalog.resolve(AssetKind::Data, filename)?;
        let bytes = self
            .backend
            .read_bytes(&loc)
            .with_context(|| format!("Reading Bincode file {loc}"))?;
        let (data, _) = bincode::decode_from_slice(
            &bytes,
            bincode::config::standard()
                .with_little_endian()
                .with_variable_int_encoding(),
        )
        .with_context(|| format!("Decoding bincode file {loc:?}"))?;
        Ok(data)
    }
}

/// Load and parse a TOML file into the specified data structure
pub fn load_toml<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Reading TOML file {}", path.display()))?;
    let data = toml::from_str::<T>(&content)
        .with_context(|| format!("Parsing TOML file {}", path.display()))?;
    Ok(data)
}
