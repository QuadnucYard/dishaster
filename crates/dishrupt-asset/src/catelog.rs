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
        if let Some(suffix) = config.default_ext_for(kind)
            && !uri.ends_with(suffix)
        {
            uri.push('.');
            uri.push_str(suffix);
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
