//! Data loading and management for Dishaster simulation

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use dishaster_core::resources::GameModelRegistry;
use dishrupt_core::model_registry::*;
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
                format!("Assets path does not exist: {}", assets_path.display()),
            )));
        }
        Ok(Self { assets_path })
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
}
