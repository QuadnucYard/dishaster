//! Integration tests for validating asset references in dishaster
//!
//! These tests ensure that all assets referenced in game data files actually exist
//! in the Godot project, preventing runtime errors from missing resources.

use std::path::{Path, PathBuf};

use anyhow::Result;
use dishaster_data::{DataLoader, GameDataAssets, load_toml};
use dishrupt_asset::{AssetCatalog, AssetKind, AssetPathConfig, AssetResolver, ResourceLocator};

/// Get the workspace root directory (where Cargo.toml with workspace is)
fn workspace_root() -> PathBuf {
    // tests/Cargo.toml is in workspace_root/tests/
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir).parent().unwrap().to_path_buf()
}

/// Get the godot project directory
fn godot_dir() -> PathBuf {
    workspace_root().join("godot")
}

/// Get the assets data directory
fn data_dir() -> PathBuf {
    workspace_root().join("assets/data")
}

/// Create asset catalog for testing
fn load_catalog() -> AssetCatalog {
    fn load_catalog_impl() -> Result<AssetCatalog> {
        let config_path = godot_dir().join("assets.toml");
        let assets_path_config = load_toml::<AssetPathConfig>(&config_path)?;
        let catalog = AssetCatalog::new(std::sync::Arc::new(assets_path_config), AssetResolver);
        Ok(catalog)
    }

    load_catalog_impl().expect("Failed to load asset catalog")
}

fn load_data() -> GameDataAssets {
    let mut loader = DataLoader::new(data_dir()).expect("Failed to create data loader");
    loader.load_all_data().expect("Failed to load game data")
}

/// Convert a res:// URI to a filesystem path for validation
fn uri_to_fs_path(uri: &str) -> PathBuf {
    let path_str = uri.strip_prefix("res://").unwrap_or(uri);
    godot_dir().join(path_str)
}

/// Helper to check and report asset existence
fn assert_asset_exists(catalog: &AssetCatalog, kind: AssetKind, id: &str, source: &str) {
    match catalog.resolve(kind, id) {
        Ok(ResourceLocator::Uri(uri)) => {
            let fs_path = uri_to_fs_path(&uri);
            assert!(
                fs_path.exists(),
                "Asset {:?} '{}' (from {}) resolved to '{}' but file does not exist at {:?}",
                kind,
                id,
                source,
                uri,
                fs_path
            );
        }
        Ok(ResourceLocator::Fs(path)) => {
            assert!(
                path.exists(),
                "Asset {:?} '{}' (from {}) resolved to filesystem path but does not exist at {:?}",
                kind,
                id,
                source,
                path
            );
        }
        Err(e) => {
            panic!(
                "Failed to resolve asset {:?} '{}' (from {}): {}",
                kind, id, source, e
            );
        }
    }
}

#[test]
fn test_management_decision_sprites_exist() {
    let catalog = load_catalog();
    let data = load_data();

    for decision in data.models.mgmt_decisions.iter() {
        assert_asset_exists(
            &catalog,
            AssetKind::Texture,
            decision.icon.path(),
            &format!("mgmt_decisions.ron::{}", decision.id),
        );
    }
}

#[test]
fn test_management_incident_sprites_exist() {
    let catalog = load_catalog();
    let data = load_data();

    for incident in data.models.mgmt_incidents.iter() {
        assert_asset_exists(
            &catalog,
            AssetKind::Texture,
            incident.icon.path(),
            &format!("mgmt_incidents.ron::{}", incident.id),
        );
    }
}

#[test]
fn test_opening_prefabs_exist() {
    let catalog = load_catalog();
    let data = load_data();

    let opening_config = &data.opening_config;

    // Verify all prefab references
    assert_asset_exists(
        &catalog,
        AssetKind::Prefab,
        opening_config.assets.food_prefab.path(),
        "opening.ron::assets.food_prefab",
    );

    assert_asset_exists(
        &catalog,
        AssetKind::Prefab,
        opening_config.assets.face_prefab.path(),
        "opening.ron::assets.face_prefab",
    );

    assert_asset_exists(
        &catalog,
        AssetKind::Prefab,
        opening_config.assets.text_prefab.path(),
        "opening.ron::assets.text_prefab",
    );
}

#[test]
fn test_music_assets_exist() {
    let catalog = load_catalog();

    // Music tracks referenced in game code (scenes/game.rs)
    let music_refs = vec![
        ("main_theme", "opening/menu"),
        ("canteen_preparation_theme", "game::PhaseMusic::Preparation"),
        ("canteen_running_theme", "game::PhaseMusic::Running"),
        ("canteen_settlement_theme", "game::PhaseMusic::Settlement"),
        ("trial_theme", "game::trial_music"),
    ];

    for (alias, source) in music_refs {
        assert_asset_exists(&catalog, AssetKind::Music, alias, source);
    }
}

#[test]
fn test_scene_assets_exist() {
    let catalog = load_catalog();

    // Scenes referenced in game code
    let scene_refs = vec![
        ("start", "scenes::DefaultSceneLoader"),
        ("game", "scenes::DefaultSceneLoader"),
    ];

    for (scene_id, source) in scene_refs {
        assert_asset_exists(&catalog, AssetKind::Scene, scene_id, source);
    }
}
