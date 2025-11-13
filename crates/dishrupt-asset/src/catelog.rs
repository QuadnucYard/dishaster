use std::{path::PathBuf, sync::Arc};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Asset domain kinds
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Gui,
    Music,
    Prefab,
    Scene,
    Sound,
    Texture,
}

/// Per-kind configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetKindConfig {
    /// Relative prefix under `res_base` or a URI base (e.g. "assets/sounds" or "pkg://bundle/audio")
    pub prefix: String,

    /// Default file extension for this kind (e.g. "ron", "ogg"). If None, callers must provide ext.
    #[serde(default)]
    pub default_ext: Option<String>,

    /// Per-kind alias map: logical id -> target id or path
    #[serde(default)]
    pub aliases: FxHashMap<String, String>,
}

/// Top-level asset path configuration (single HashMap per-kind)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPathConfig {
    // /// Base path that replaces "res://" or the root of relative asset lookups.
    // pub res_base: String,
    /// Per-kind configuration map. Keyed by AssetKind for clarity & grouping.
    #[serde(default)]
    pub kinds: FxHashMap<AssetKind, AssetKindConfig>,

    /// Optional global aliases that apply across all kinds (fallback).
    #[serde(default)]
    pub global_aliases: FxHashMap<String, String>,
}

impl AssetPathConfig {
    /// Helper: get per-kind config (None if the kind isn't configured)
    pub fn kind_config(&self, kind: AssetKind) -> Option<&AssetKindConfig> {
        self.kinds.get(&kind)
    }

    /// Resolve alias with per-kind precedence, falling back to global aliases.
    /// Returns the mapped target if present.
    pub fn resolve_alias<'a>(&'a self, kind: AssetKind, id: &'a str) -> Option<&'a str> {
        if let Some(kind_cfg) = self.kinds.get(&kind)
            && let Some(target) = kind_cfg.aliases.get(id)
        {
            return Some(target.as_str());
        }
        self.global_aliases.get(id).map(|s| s.as_str())
    }

    /// Convenience: get prefix for kind
    pub fn prefix_for(&self, kind: AssetKind) -> Option<&str> {
        self.kinds.get(&kind).map(|k| k.prefix.as_str())
    }

    /// Convenience: get default ext for kind
    pub fn default_ext_for(&self, kind: AssetKind) -> Option<&str> {
        self.kinds.get(&kind).and_then(|k| k.default_ext.as_deref())
    }
}

/// Normalized locator returned by the resolver
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceLocator {
    /// File system path
    Fs(PathBuf),
    /// URI (e.g., "res://...", etc.)
    Uri(String),
}

/// Asset resolver for locating assets based on kind and id
#[derive(Debug, Clone)]
pub struct AssetResolver;

impl AssetResolver {
    /// Generic resolver: returns a ResourceLocator normalized (no `..` traversal).
    #[allow(clippy::only_used_in_recursion)]
    pub fn resolve_with_config(
        &self,
        config: &AssetPathConfig,
        kind: AssetKind,
        id: &str,
    ) -> Result<ResourceLocator, ResolveError> {
        // 1) Already a URI or absolute -> return Uri or Fs
        if id.starts_with('/') {
            return Ok(ResourceLocator::Fs(PathBuf::from(id)));
        } else if id.contains("://") {
            return Ok(ResourceLocator::Uri(id.to_string()));
        }

        // 2) overrides (per-level) first
        if let Some(v) = config.resolve_alias(kind, id) {
            return self.resolve_with_config(config, kind, v);
        }

        // 3) prefix join: if prefix string looks like a URI base (contains "://"),
        //    treat result as a URI (concatenate with '/'), otherwise produce Fs path.
        let prefix = config.prefix_for(kind).ok_or(ResolveError::NoPrefix)?;

        // ensure there's exactly one slash between prefix and id
        let mut uri = prefix.to_string();
        if !prefix.ends_with('/') {
            uri.push('/');
        }
        uri.push_str(id.strip_prefix('/').unwrap_or(id));

        // Add default extension only if the path doesn't already have any extension
        if let Some(suffix) = config.default_ext_for(kind) {
            // Check if the path already has an extension (contains a dot after the last slash)
            let has_extension = uri
                .rsplit_once('/')
                .map_or_else(|| uri.contains('.'), |(_, filename)| filename.contains('.'));

            if !has_extension {
                uri.push('.');
                uri.push_str(suffix);
            }
        }

        self.resolve_with_config(config, kind, &uri)
    }
}

/// Errors that can occur during asset resolution
#[derive(Debug, Error)]
pub enum ResolveError {
    /// No prefix defined for the given asset kind
    #[error("no prefix defined for asset kind")]
    NoPrefix,
}

/// Asset catalog for resolving asset paths
#[derive(Debug, Clone)]
pub struct AssetCatalog {
    cfg: Arc<AssetPathConfig>,
    resolver: AssetResolver,
}

impl AssetCatalog {
    /// Create a new asset catalog with the given configuration and resolver
    pub fn new(cfg: Arc<AssetPathConfig>, resolver: AssetResolver) -> Self {
        Self { cfg, resolver }
    }

    /// Resolve an asset by kind and id
    pub fn resolve(&self, kind: AssetKind, id: &str) -> Result<ResourceLocator, ResolveError> {
        // use the pure resolver against the config
        let loc = self.resolver.resolve_with_config(&self.cfg, kind, id)?;
        Ok(loc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! hash_map {
        () => {{
            rustc_hash::FxHashMap::default()
        }};

        ( $( $key:expr => $value:expr ),* $(,)? ) => {{
            let mut map = rustc_hash::FxHashMap::default();
            $( map.insert($key, $value); )*
            map
        }}
    }

    fn create_test_config() -> AssetPathConfig {
        AssetPathConfig {
            kinds: hash_map! {
                AssetKind::Music => AssetKindConfig {
                    prefix: "res://assets/music".to_string(),
                    default_ext: None,
                    aliases: hash_map! {
                    "main_theme".to_string() => "main_theme.ogg".to_string(),
                    "battle_theme".to_string() => "battle.ogg".to_string(),
                },
                },
                AssetKind::Scene => AssetKindConfig {
                    prefix: "res://assets/scenes".to_string(),
                    default_ext: Some("tscn".to_string()),
                    aliases: hash_map! {},
                },
                AssetKind::Texture => AssetKindConfig {
                    prefix: "res://assets/sprites".to_string(),
                    default_ext: None,
                    aliases: hash_map! {},
                },
            },
            global_aliases: hash_map! {},
        }
    }

    fn create_test_catalog() -> AssetCatalog {
        let config = create_test_config();
        create_catalog(config)
    }

    fn create_catalog(config: AssetPathConfig) -> AssetCatalog {
        let resolver = AssetResolver;
        AssetCatalog::new(Arc::new(config), resolver)
    }

    #[test]
    fn test_resolve_alias() {
        let catalog = create_test_catalog();

        // Test music alias resolution
        let result = catalog.resolve(AssetKind::Music, "main_theme").unwrap();
        assert_eq!(
            result,
            ResourceLocator::Uri("res://assets/music/main_theme.ogg".to_string())
        );

        let result = catalog.resolve(AssetKind::Music, "battle_theme").unwrap();
        assert_eq!(
            result,
            ResourceLocator::Uri("res://assets/music/battle.ogg".to_string())
        );
    }

    #[test]
    fn test_resolve_with_absolute_uri() {
        let catalog = create_test_catalog();

        // Already a URI - should return as-is
        let result = catalog
            .resolve(AssetKind::Music, "res://assets/music/custom.ogg")
            .unwrap();
        assert_eq!(
            result,
            ResourceLocator::Uri("res://assets/music/custom.ogg".to_string())
        );
    }

    #[test]
    fn test_resolve_with_default_extension() {
        let catalog = create_test_catalog();

        // Scene without extension - should add .tscn
        let result = catalog.resolve(AssetKind::Scene, "main_menu").unwrap();
        assert_eq!(
            result,
            ResourceLocator::Uri("res://assets/scenes/main_menu.tscn".to_string())
        );

        // Scene with extension already - should not double-add
        let result = catalog.resolve(AssetKind::Scene, "game.tscn").unwrap();
        assert_eq!(
            result,
            ResourceLocator::Uri("res://assets/scenes/game.tscn".to_string())
        );

        // Scene with different extension - should not add default extension
        let result = catalog.resolve(AssetKind::Scene, "game.json").unwrap();
        assert_eq!(
            result,
            ResourceLocator::Uri("res://assets/scenes/game.json".to_string())
        );
    }

    #[test]
    fn test_resolve_without_default_extension() {
        let catalog = create_test_catalog();

        // Texture has no default extension - should use as-is
        let result = catalog.resolve(AssetKind::Texture, "player.png").unwrap();
        assert_eq!(
            result,
            ResourceLocator::Uri("res://assets/sprites/player.png".to_string())
        );
    }

    #[test]
    fn test_resolve_relative_path() {
        let catalog = create_test_catalog();

        // Relative path joins with prefix
        let result = catalog
            .resolve(AssetKind::Texture, "characters/hero.png")
            .unwrap();
        assert_eq!(
            result,
            ResourceLocator::Uri("res://assets/sprites/characters/hero.png".to_string())
        );
    }

    #[test]
    fn test_resolve_absolute_filesystem_path() {
        let catalog = create_test_catalog();

        // Absolute path starting with / - should return Fs
        let result = catalog.resolve(AssetKind::Music, "/tmp/sound.ogg").unwrap();
        assert_eq!(result, ResourceLocator::Fs(PathBuf::from("/tmp/sound.ogg")));
    }

    #[test]
    fn test_resolve_no_prefix_error() {
        let config = AssetPathConfig {
            kinds: hash_map! {},
            global_aliases: hash_map! {},
        };
        let catalog = create_catalog(config);

        // Should error when kind has no prefix configured
        let result = catalog.resolve(AssetKind::Music, "test.ogg");
        assert!(matches!(result, Err(ResolveError::NoPrefix)));
    }

    #[test]
    fn test_prefix_trailing_slash_normalization() {
        let config = AssetPathConfig {
            kinds: hash_map! {
                AssetKind::Sound => AssetKindConfig {
                    prefix: "res://sounds/".to_string(), // with trailing slash
                    default_ext: Some("wav".to_string()),
                    aliases: hash_map! {},
                },
            },
            global_aliases: hash_map! {},
        };
        let catalog = create_catalog(config);

        // Should handle trailing slash correctly - no double slash
        let result = catalog.resolve(AssetKind::Sound, "jump").unwrap();
        assert_eq!(
            result,
            ResourceLocator::Uri("res://sounds/jump.wav".to_string())
        );
    }

    #[test]
    fn test_global_aliases() {
        let config = AssetPathConfig {
            kinds: hash_map! {
                AssetKind::Texture => AssetKindConfig {
                    prefix: "res://assets".to_string(),
                    default_ext: None,
                    aliases: hash_map! {},
                },
            },
            global_aliases: hash_map! {
                "common_icon".to_string() => "ui/icon.png".to_string(),
            },
        };
        let catalog = create_catalog(config);

        // Should resolve global alias
        let result = catalog.resolve(AssetKind::Texture, "common_icon").unwrap();
        assert_eq!(
            result,
            ResourceLocator::Uri("res://assets/ui/icon.png".to_string())
        );
    }

    #[test]
    fn test_kind_alias_precedence_over_global() {
        let config = AssetPathConfig {
            kinds: hash_map! {
                AssetKind::Music => AssetKindConfig {
                    prefix: "res://audio".to_string(),
                    default_ext: None,
                    aliases: hash_map! {
                        "theme".to_string() => "music_theme.ogg".to_string(),
                    },
                },
            },
            global_aliases: hash_map! {
                "theme".to_string() => "global_theme.ogg".to_string(),
            },
        };
        let catalog = create_catalog(config);

        // Kind-specific alias should take precedence over global
        let result = catalog.resolve(AssetKind::Music, "theme").unwrap();
        assert_eq!(
            result,
            ResourceLocator::Uri("res://audio/music_theme.ogg".to_string())
        );
    }
}
