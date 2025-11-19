#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Get the workspace root directory (where Cargo.toml with workspace is)
pub fn workspace_root() -> PathBuf {
    // tests/Cargo.toml is in workspace_root/tests/
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir).parent().unwrap().to_path_buf()
}

/// Get the godot project directory
pub fn godot_dir() -> PathBuf {
    workspace_root().join("godot")
}

/// Get the assets data directory
pub fn data_dir() -> PathBuf {
    workspace_root().join("assets/data")
}
