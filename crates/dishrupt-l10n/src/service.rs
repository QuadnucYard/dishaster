use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use fluent_bundle::{FluentResource, concurrent::FluentBundle};
use fluent_loader::{DummyProvider, FluentProvider};
use fluent_templates::ArcLoader;
use unic_langid::LanguageIdentifier;

use crate::builtins;

/// A localization service that holds the current language and localization loader.
pub struct L10nService {
    lang: RwLock<LanguageIdentifier>,
    locales: ArcSwap<Box<dyn FluentProvider>>,
}

impl L10nService {
    /// Create a new empty `L10nService` with the specified language.
    pub fn new_empty() -> Self {
        Self {
            lang: RwLock::new(Default::default()),
            locales: ArcSwap::new(Arc::new(Box::new(DummyProvider))),
        }
    }

    /// Create a new `L10nService` with the specified language.
    pub fn new_with_lang(fallback_lang: LanguageIdentifier) -> Self {
        Self {
            lang: RwLock::new(fallback_lang.clone()),
            locales: ArcSwap::new(Arc::new(Box::new(DummyProvider))),
        }
    }

    /// Get the current language identifier.
    pub fn get_lang(&self) -> LanguageIdentifier {
        // Handle poisoned lock by retrieving the inner guard instead of panicking.
        match self.lang.read() {
            Ok(guard) => guard.clone(),
            Err(poison) => poison.into_inner().clone(),
        }
    }

    /// Get the global localization loader.
    pub fn get_locales(&self) -> Arc<Box<dyn FluentProvider>> {
        self.locales.load_full()
    }

    /// Set the global localization loader.
    pub fn set_locales(&self, loader: impl FluentProvider + 'static) {
        self.locales.store(Arc::new(Box::new(loader)));
    }
}

/// Build an `ArcLoader` with custom settings.
pub fn build_arc_loader(path: &str, fallback_lang: LanguageIdentifier) -> ArcLoader {
    ArcLoader::builder(path, fallback_lang)
        .customize(default_customizer)
        .build()
        .expect("build ArcLoader with custom path")
}

/// Default customizer for Fluent bundles.
pub fn default_customizer(bundle: &mut FluentBundle<Arc<FluentResource>>) {
    bundle.set_use_isolating(false);
    add_builtins(bundle);
}

/// Add builtin functions to a Fluent bundle.
fn add_builtins(bundle: &mut FluentBundle<Arc<FluentResource>>) {
    // The builtin function in v0.16.1 are missing, so we have to implement them by ourself.
    bundle
        .add_function("NUM", builtins::number)
        .expect("Failed to add function `NUM` to the bundle.");
    bundle
        .add_function("PCT", builtins::percent)
        .expect("Failed to add function `PCT` to the bundle.");
}
