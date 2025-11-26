//! Filesystem backend implementation for `DataBackend`.

use std::path::PathBuf;

use anyhow::bail;

use crate::{
    ResourceLocator,
    backend::{DataBackend, LoadError},
};

/// Filesystem-based resource backend rooted at a path.
pub struct FsBackend {
    root: PathBuf,
}

impl FsBackend {
    /// Create a new [`FsBackend`] with the given root path.
    ///
    /// Returns an error if the root path does not exist.
    pub fn new(root: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let root = root.into();
        if !std::fs::exists(&root).is_ok_and(|exists| exists) {
            bail!(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Assets path does not exist: {}. The current working directory is: {:?}",
                    root.display(),
                    std::env::current_dir()
                ),
            ));
        }
        Ok(Self { root })
    }
}

impl DataBackend for FsBackend {
    fn exists(&self, locator: &ResourceLocator) -> Result<bool, LoadError> {
        let path = fs_loc(locator)?;
        let full_path = self.root.join(path);
        Ok(full_path.exists())
    }

    fn is_file(&self, locator: &ResourceLocator) -> Result<bool, LoadError> {
        let path = fs_loc(locator)?;
        let full_path = self.root.join(path);
        Ok(full_path.is_file())
    }

    fn is_dir(&self, locator: &ResourceLocator) -> Result<bool, LoadError> {
        let path = fs_loc(locator)?;
        let path = self.root.join(path);
        Ok(path.is_dir())
    }

    fn list_dir(&self, locator: &ResourceLocator) -> Result<Vec<ResourceLocator>, LoadError> {
        let path = fs_loc(locator)?;
        let full_path = self.root.join(path);
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&full_path)? {
            let entry = entry?;
            entries.push(ResourceLocator::Fs(path.join(entry.file_name())));
        }
        Ok(entries)
    }

    fn read_bytes(&self, locator: &ResourceLocator) -> Result<Vec<u8>, LoadError> {
        let path = fs_loc(locator)?;
        let full_path = self.root.join(path);
        std::fs::read(&full_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                LoadError::NotFound(ResourceLocator::Fs(full_path))
            } else {
                anyhow::Error::new(e).into()
            }
        })
    }
}

fn fs_loc(locator: &ResourceLocator) -> Result<&PathBuf, LoadError> {
    let ResourceLocator::Fs(path) = locator else {
        return Err(LoadError::UnsupportedLocation(locator.clone()));
    };
    Ok(path)
}
