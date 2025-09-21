use core::fmt;

use serde::Deserialize;

use crate::prelude::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Deserialize)]
pub struct PrefabReference(EcoString);

impl PrefabReference {
    pub fn new(path: impl Into<EcoString>) -> Self {
        Self(path.into())
    }

    pub fn path(&self) -> &EcoString {
        &self.0
    }

    pub fn stem(&self) -> &str {
        self.0.rsplit('/').next().unwrap()
    }

    pub fn parent(&self) -> &str {
        if let Some(i) = self.0.rfind('/') {
            &self.0[..i]
        } else {
            ""
        }
    }
}

impl fmt::Display for PrefabReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Deserialize)]
pub struct SpriteReference(EcoString);

impl SpriteReference {
    pub fn new(path: impl Into<EcoString>) -> Self {
        Self(path.into())
    }

    pub fn path(&self) -> &EcoString {
        &self.0
    }
}

impl fmt::Display for SpriteReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Deserialize)]
pub struct SoundReference(EcoString);

impl SoundReference {
    pub fn new(path: impl Into<EcoString>) -> Self {
        Self(path.into())
    }

    pub fn path(&self) -> &EcoString {
        &self.0
    }
}

impl fmt::Display for SoundReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
