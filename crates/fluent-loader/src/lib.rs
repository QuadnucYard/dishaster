//! Synchronous Fluent loader and re-implementation of `fluent_templates` utilities

pub mod error;
pub mod lookup;

use std::{borrow::Cow, collections::HashMap};

use fluent_bundle::FluentValue;
use fluent_templates::Loader;
pub use fluent_templates::loader::build_fallbacks;
use unic_langid::LanguageIdentifier;

/// A loader capable of looking up Fluent keys given a language.
pub trait FluentProvider: Send + Sync {
    /// Look up `text_id` for `lang` in Fluent.
    fn lookup(&self, lang: &LanguageIdentifier, text_id: &str) -> String {
        self.lookup_complete(lang, text_id, None)
    }

    /// Look up `text_id` for `lang` with `args` in Fluent.
    fn lookup_with_args(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: &HashMap<Cow<'static, str>, FluentValue>,
    ) -> String {
        self.lookup_complete(lang, text_id, Some(args))
    }

    /// Look up `text_id` for `lang` in Fluent, using any `args` if provided.
    fn lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> String;

    /// Look up `text_id` for `lang` in Fluent.
    fn try_lookup(&self, lang: &LanguageIdentifier, text_id: &str) -> Option<String> {
        self.try_lookup_complete(lang, text_id, None)
    }

    /// Look up `text_id` for `lang` with `args` in Fluent.
    fn try_lookup_with_args(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: &HashMap<Cow<'static, str>, FluentValue>,
    ) -> Option<String> {
        self.try_lookup_complete(lang, text_id, Some(args))
    }

    /// Look up `text_id` for `lang` in Fluent, using any `args` if provided.
    fn try_lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> Option<String>;

    /// Returns an Iterator over the locales that are present.
    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_>;
}

impl<T> FluentProvider for T
where
    T: Loader + Send + Sync,
{
    fn lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> String {
        <Self as Loader>::lookup_complete(self, lang, text_id, args)
    }

    fn try_lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> Option<String> {
        <Self as Loader>::try_lookup_complete(self, lang, text_id, args)
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        <Self as Loader>::locales(self)
    }
}

/// A dummy Fluent provider that returns the text id as-is.
pub struct DummyProvider;

impl FluentProvider for DummyProvider {
    fn lookup_complete(
        &self,
        _lang: &LanguageIdentifier,
        text_id: &str,
        _args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> String {
        text_id.to_string()
    }

    fn try_lookup_complete(
        &self,
        _lang: &LanguageIdentifier,
        text_id: &str,
        _args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> Option<String> {
        Some(text_id.to_string())
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        Box::new(std::iter::empty())
    }
}
