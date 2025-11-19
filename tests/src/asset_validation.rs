//! Integration tests for validating asset references in dishaster
//!
//! These tests ensure that all assets referenced in game data files actually exist
//! in the Godot project, preventing runtime errors from missing resources.

mod harness;

use std::path::PathBuf;

use anyhow::Result;
use dishaster_data::{DataLoader, GameDataAssets, load_toml};
use dishrupt_asset::{AssetCatalog, AssetKind, AssetPathConfig, AssetResolver, ResourceLocator};
use harness::{data_dir, godot_dir};

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
fn check_asset_exists(
    catalog: &AssetCatalog,
    kind: AssetKind,
    id: &str,
    source: &str,
) -> Result<(), String> {
    match catalog.resolve(kind, id) {
        Ok(ResourceLocator::Uri(uri)) => {
            let fs_path = uri_to_fs_path(&uri);
            if !fs_path.exists() {
                return Err(format!(
                    "Asset {:?} '{}' (from {}) resolved to '{}' but file does not exist at {:?}",
                    kind, id, source, uri, fs_path
                ));
            }
        }
        Ok(ResourceLocator::Fs(path)) => {
            if !path.exists() {
                return Err(format!(
                    "Asset {:?} '{}' (from {}) resolved to filesystem path but does not exist at {:?}",
                    kind, id, source, path
                ));
            }
        }
        Err(e) => {
            return Err(format!(
                "Failed to resolve asset {:?} '{}' (from {}): {}",
                kind, id, source, e
            ));
        }
    }
    Ok(())
}

/// Collect all errors from asset checks and panic if any are found
fn assert_no_errors(errors: Vec<String>, asset_type: &str) {
    if !errors.is_empty() {
        panic!(
            "Found {} missing {}:\n{}",
            errors.len(),
            asset_type,
            errors.join("\n")
        );
    }
}

#[test]
fn test_dish_prefabs_exist() {
    let catalog = load_catalog();
    let data = load_data();

    let errors: Vec<_> = (data.models.dishes.iter())
        .filter_map(|dish| {
            check_asset_exists(
                &catalog,
                AssetKind::Prefab,
                dish.display.res.path(),
                &format!("dishes.ron::{}", dish.id),
            )
            .err()
        })
        .collect();

    assert_no_errors(errors, "dish prefabs");
}

#[test]
fn test_management_decision_sprites_exist() {
    let catalog = load_catalog();
    let data = load_data();

    let errors: Vec<_> = (data.models.mgmt_decisions.iter())
        .filter_map(|decision| {
            check_asset_exists(
                &catalog,
                AssetKind::Texture,
                decision.icon.path(),
                &format!("mgmt_decisions.ron::{}", decision.id),
            )
            .err()
        })
        .collect();

    assert_no_errors(errors, "management decision sprites");
}

#[test]
fn test_management_incident_sprites_exist() {
    let catalog = load_catalog();
    let data = load_data();

    let errors: Vec<_> = (data.models.mgmt_incidents.iter())
        .filter_map(|incident| {
            check_asset_exists(
                &catalog,
                AssetKind::Texture,
                incident.icon.path(),
                &format!("mgmt_incidents.ron::{}", incident.id),
            )
            .err()
        })
        .collect();

    assert_no_errors(errors, "management incident sprites");
}

#[test]
fn test_opening_prefabs_exist() {
    let catalog = load_catalog();
    let data = load_data();

    let opening_config = &data.opening_config;

    let checks = [
        (
            opening_config.assets.food_prefab.path(),
            "opening.ron::assets.food_prefab",
        ),
        (
            opening_config.assets.face_prefab.path(),
            "opening.ron::assets.face_prefab",
        ),
        (
            opening_config.assets.text_prefab.path(),
            "opening.ron::assets.text_prefab",
        ),
    ];

    let errors: Vec<_> = (checks.iter())
        .filter_map(|(path, source)| {
            check_asset_exists(&catalog, AssetKind::Prefab, path, source).err()
        })
        .collect();

    assert_no_errors(errors, "opening prefabs");
}

#[test]
fn test_music_assets_exist() {
    let catalog = load_catalog();

    // Music tracks referenced in game code (scenes/game.rs)
    let music_refs = [
        ("main_theme", "opening/menu"),
        ("canteen_preparation_theme", "game::PhaseMusic::Preparation"),
        ("canteen_running_theme", "game::PhaseMusic::Running"),
        ("canteen_settlement_theme", "game::PhaseMusic::Settlement"),
        ("trial_theme", "game::trial_music"),
    ];

    let errors: Vec<_> = (music_refs.iter())
        .filter_map(|(alias, source)| {
            check_asset_exists(&catalog, AssetKind::Music, alias, source).err()
        })
        .collect();

    assert_no_errors(errors, "music assets");
}

#[test]
fn test_scene_assets_exist() {
    let catalog = load_catalog();

    // Scenes referenced in game code
    let scene_refs = [
        ("start", "scenes::DefaultSceneLoader"),
        ("game", "scenes::DefaultSceneLoader"),
    ];

    let errors: Vec<_> = (scene_refs.iter())
        .filter_map(|(scene_id, source)| {
            check_asset_exists(&catalog, AssetKind::Scene, scene_id, source).err()
        })
        .collect();

    assert_no_errors(errors, "scene assets");
}
