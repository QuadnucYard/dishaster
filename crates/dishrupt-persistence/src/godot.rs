//! Godot user://-based storage backend.

use std::path::Path;

use anyhow::{Context, Result, anyhow};
use godot::{
    classes::{FileAccess, file_access::ModeFlags},
    prelude::*,
};

use crate::PersistentStorage;

/// Godot user://-based storage backend.
pub struct GodotUserStorage;

impl PersistentStorage for GodotUserStorage {
    fn read(&self, path: &str) -> Result<Option<Vec<u8>>> {
        let file_path = Path::new("user://").join(path);
        let path_str = file_path
            .to_str()
            .with_context(|| format!("invalid path string: {:?}", file_path))?;
        if !FileAccess::file_exists(path_str) {
            return Ok(None);
        }

        let bytes = FileAccess::get_file_as_bytes(path_str);
        Ok(Some(bytes.to_vec()))
    }

    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<()> {
        let file_path = Path::new("user://").join(path);
        let path_str = file_path
            .to_str()
            .with_context(|| format!("invalid path string: {:?}", file_path))?;

        let mut file = FileAccess::open(path_str, ModeFlags::WRITE)
            .with_context(|| format!("failed to open file: {}", path_str))?;

        let packed = PackedArray::from(bytes);
        if !file.store_buffer(&packed) {
            return Err(anyhow!("failed to write buffer to file: {}", path_str));
        }
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<()> {
        let file_path = Path::new("user://").join(path);
        let path_str = file_path
            .to_str()
            .with_context(|| format!("invalid path string: {:?}", file_path))?;

        if FileAccess::file_exists(path_str) {
            let mut da = godot::classes::DirAccess::open("user://")
                .with_context(|| "failed to open user:// directory")?;
            let err = da.remove(path);
            if err != godot::global::Error::OK {
                return Err(anyhow!(
                    "failed to delete file: {path_str} (error: {err:?})"
                ));
            }
        }
        Ok(())
    }
}
