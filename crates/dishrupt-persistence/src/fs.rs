//! Filesystem-based persistent storage backend.

use std::{fs, io::ErrorKind, path::PathBuf};

use anyhow::{Context, Result};

use crate::{Persistable, PersistentStorage};

/// Filesystem-based storage backend.
pub struct FsStorage {
    root_dir: PathBuf,
}

impl FsStorage {
    /// Create a new filesystem storage backend rooted at the specified directory.
    pub fn new(root_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root_dir)
            .with_context(|| format!("failed to create persistence dir {:?}", root_dir))?;
        Ok(Self { root_dir })
    }
}

impl PersistentStorage for FsStorage {
    fn load_or_create<T: Persistable>(
        &mut self,
        path: &str,
        init: impl FnOnce() -> T,
    ) -> Result<T> {
        let file_path = self.root_dir.join(path);
        if !file_path.exists() {
            // Initialize new data and save it.
            let value = init();
            self.save(path, &value)?;
            return Ok(value);
        }

        let data = fs::read(&file_path)?;
        T::from_bytes(data)
    }

    fn save<T: Persistable>(&mut self, path: &str, data: &T) -> Result<()> {
        let file_path = self.root_dir.join(path);

        // Write to a temporary file first.
        let tmp = file_path.with_extension("tmp");
        fs::write(&tmp, &data.to_bytes()?)
            .with_context(|| format!("failed to write temp save {tmp:?}"))?;

        // Replace the old save file with the new one.
        match fs::rename(&tmp, &file_path) {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == ErrorKind::PermissionDenied => {
                if file_path.exists() {
                    fs::remove_file(&file_path).with_context(|| {
                        format!("failed to remove old save before replacing {file_path:?}")
                    })?;
                }
                fs::rename(&tmp, &file_path)
                    .with_context(|| format!("failed to replace save file {file_path:?}"))?;
                Ok(())
            }
            Err(err) => Err(err)
                .with_context(|| format!("failed to move temp save {tmp:?} -> {file_path:?}")),
        }
    }
}
