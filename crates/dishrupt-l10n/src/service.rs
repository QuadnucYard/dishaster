//! Localization utilities for Fluent-based translations.
//!
//! Lightweight, thread-safe service for the current language and provider.

use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use fluent_bundle::{FluentResource, concurrent::FluentBundle};
use fluent_loader::{DummyProvider, FluentProvider};
use fluent_templates::ArcLoader;
use unic_langid::LanguageIdentifier;

use crate::builtins;

/// Thread-safe localization service holding the active language and provider.
pub struct L10nService {
    lang: RwLock<LanguageIdentifier>,
    locales: ArcSwap<Box<dyn FluentProvider>>,
}

impl L10nService {
    /// Create an empty service with default language and `DummyProvider`.
    pub fn new_empty() -> Self {
        Self {
            lang: RwLock::new(Default::default()),
            locales: ArcSwap::new(Arc::new(Box::new(DummyProvider))),
        }
    }

    /// Create a service initialized with the given language.
    pub fn new_with_lang(lang: LanguageIdentifier) -> Self {
        Self {
            lang: RwLock::new(lang.clone()),
            locales: ArcSwap::new(Arc::new(Box::new(DummyProvider))),
        }
    }

    /// Return the current language.
    pub fn get_lang(&self) -> LanguageIdentifier {
        match self.lang.read() {
            Ok(guard) => guard.clone(),
            Err(poison) => poison.into_inner().clone(),
        }
    }

    /// Return an `Arc` to the current `FluentProvider`.
    pub fn get_locales(&self) -> Arc<Box<dyn FluentProvider>> {
        self.locales.load_full()
    }

    /// Replace the global `FluentProvider` atomically.
    pub fn set_locales(&self, loader: impl FluentProvider + 'static) {
        self.locales.store(Arc::new(Box::new(loader)));
    }
}

/// Build an `ArcLoader` using the `default_customizer`.
pub fn build_arc_loader(path: &str, fallback_lang: LanguageIdentifier) -> ArcLoader {
    ArcLoader::builder(path, fallback_lang)
        .customize(default_customizer)
        .build()
        .expect("build ArcLoader with custom path")
}

/// Customize Fluent bundles and register required builtin functions.
pub fn default_customizer(bundle: &mut FluentBundle<Arc<FluentResource>>) {
    bundle.set_use_isolating(false);
    add_builtins(bundle);
}

/// Register `NUM` and `PCT` builtin functions on the bundle.
fn add_builtins(bundle: &mut FluentBundle<Arc<FluentResource>>) {
    // The builtin function in v0.16.1 are missing, so we have to implement them by ourself.
    bundle
        .add_function("NUM", builtins::number)
        .expect("Failed to add function `NUM` to the bundle.");
    bundle
        .add_function("PCT", builtins::percent)
        .expect("Failed to add function `PCT` to the bundle.");
}
