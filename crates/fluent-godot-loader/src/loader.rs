use std::{borrow::Cow, collections::HashMap, sync::Arc};

use fluent_bundle::{FluentResource, FluentValue, concurrent::FluentBundle};
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use fluent_loader::{
    SyncLoader, build_fallbacks,
    error::{FluentError, LoaderError, LookupError},
    lookup::{lookup_no_default_fallback, lookup_single_language},
};
use godot::{
    classes::{ResourceLoader, resource_loader::CacheMode},
    prelude::*,
};
use unic_langid::LanguageIdentifier;

use crate::resource::FluentResource as GodotFluentResource;

type LoaderResult<T> = std::result::Result<T, LoaderError>;

type Customize = Option<Box<dyn FnMut(&mut FluentBundle<Arc<FluentResource>>)>>;

/// A builder for [`GodotResLoader`].
pub struct GodotResLoaderBuilder<'a, 'b> {
    location: &'a str,
    fallback: LanguageIdentifier,
    shared: Option<&'b [String]>,
    customize: Customize,
}

impl<'a, 'b> GodotResLoaderBuilder<'a, 'b> {
    /// Adds Fluent resources that are shared across all localizations.
    pub fn shared_resources<'b2>(
        self,
        shared: Option<&'b2 [String]>,
    ) -> GodotResLoaderBuilder<'a, 'b2> {
        GodotResLoaderBuilder {
            location: self.location,
            fallback: self.fallback,
            shared,
            customize: self.customize,
        }
    }

    /// Allows you to customise each `FluentBundle`.
    pub fn customize(
        mut self,
        customize: impl FnMut(&mut FluentBundle<Arc<FluentResource>>) + 'static,
    ) -> Self {
        self.customize = Some(Box::new(customize));
        self
    }

    /// Constructs an [`GodotResLoader`] from the settings provided.
    pub fn build(mut self) -> Result<GodotResLoader, Box<dyn std::error::Error>> {
        let mut resources = HashMap::new();

        let mut loader = ResourceLoader::singleton();
        let location = self.location.to_godot();
        for entry in loader.list_directory(self.location).as_slice() {
            if entry.ends_with("/") {
                let lang = entry.trim_suffix("/").to_string();
                let lang_resources = read_from_dir(&location.path_join(entry), &mut loader)?
                    .into_iter()
                    .map(Arc::new)
                    .collect::<Vec<_>>();
                resources.insert(lang.parse::<LanguageIdentifier>()?, lang_resources);
            }
        }

        let mut bundles = HashMap::new();
        for (lang, v) in resources.iter() {
            let mut bundle = FluentBundle::new_concurrent(vec![lang.clone()]);

            for shared_resource in self.shared.unwrap_or(&[]) {
                bundle
                    .add_resource(Arc::new(read_from_file(shared_resource, &mut loader)?))
                    .map_err(|errors| LoaderError::FluentBundle { errors })?;
            }

            for res in v {
                bundle
                    .add_resource(res.clone())
                    .map_err(|errors| LoaderError::FluentBundle { errors })?;
            }

            if let Some(customize) = self.customize.as_mut() {
                (customize)(&mut bundle);
            }

            bundles.insert(lang.clone(), bundle);
        }

        let fallbacks = build_fallbacks(&resources.keys().cloned().collect::<Vec<_>>());

        Ok(GodotResLoader {
            bundles,
            fallbacks,
            fallback: self.fallback,
        })
    }
}

fn read_from_file(path: &str, loader: &mut ResourceLoader) -> LoaderResult<FluentResource> {
    let res = loader
        .load_ex(path)
        .type_hint(GodotFluentResource::RES_TYPE)
        .cache_mode(CacheMode::IGNORE)
        .done()
        .and_then(|res| res.try_cast::<GodotFluentResource>().ok())
        .ok_or_else(|| LoaderError::NotFound {
            path: path.to_string(),
        })?;
    let text = res.bind().text().to_string();
    resource_from_str(&text)
}

fn read_from_dir(path: &GString, loader: &mut ResourceLoader) -> LoaderResult<Vec<FluentResource>> {
    let mut srcs = Vec::new();

    for entry in loader
        .list_directory(path)
        .as_slice()
        .iter()
        .filter(|e| e.ends_with(".ftl"))
    {
        let full_path = path.path_join(entry);
        if let Ok(string) = read_from_file(&full_path.to_string(), loader) {
            srcs.push(string);
        } else {
            godot_warn!("Couldn't read {}", full_path);
        }
    }

    Ok(srcs)
}

fn resource_from_str(src: &str) -> LoaderResult<FluentResource> {
    FluentResource::try_new(src.to_owned()).map_err(|(_, errs)| FluentError::from(errs).into())
}

/// A Fluent resource loader that loads resources from `.ftl` files in Godot `res://`.
pub struct GodotResLoader {
    bundles: HashMap<LanguageIdentifier, FluentBundle<Arc<FluentResource>>>,
    fallback: LanguageIdentifier,
    fallbacks: HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
}

impl GodotResLoader {
    /// Creates a new `ArcLoaderBuilder`
    pub fn builder<'a>(
        location: &'a str,
        fallback: LanguageIdentifier,
    ) -> GodotResLoaderBuilder<'a, 'static> {
        GodotResLoaderBuilder {
            location,
            fallback,
            shared: None,
            customize: None,
        }
    }

    /// Convenience function to look up a string for a single language
    pub fn lookup_single_language<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> Result<String, LookupError> {
        lookup_single_language(&self.bundles, lang, text_id, args)
    }

    /// Convenience function to look up a string without falling back to the
    /// default fallback language
    pub fn lookup_no_default_fallback<S: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<S, FluentValue>>,
    ) -> Option<String> {
        lookup_no_default_fallback(&self.bundles, &self.fallbacks, lang, text_id, args)
    }

    /// Return the fallback language
    pub fn fallback(&self) -> &LanguageIdentifier {
        &self.fallback
    }
}

impl SyncLoader for GodotResLoader {
    // Traverse the fallback chain,
    fn lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> String {
        for lang in negotiate_languages(
            &[lang],
            &self.bundles.keys().collect::<Vec<_>>(),
            None,
            NegotiationStrategy::Filtering,
        ) {
            if let Ok(val) = self.lookup_single_language(lang, text_id, args) {
                return val;
            }
        }
        if *lang != self.fallback
            && let Ok(val) = self.lookup_single_language(&self.fallback, text_id, args)
        {
            return val;
        }
        format!("Unknown localization key: {text_id:?}")
    }

    // Traverse the fallback chain,
    fn try_lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> Option<String> {
        for lang in negotiate_languages(
            &[lang],
            &self.bundles.keys().collect::<Vec<_>>(),
            None,
            NegotiationStrategy::Filtering,
        ) {
            if let Ok(val) = self.lookup_single_language(lang, text_id, args) {
                return Some(val);
            }
        }
        if *lang != self.fallback
            && let Ok(val) = self.lookup_single_language(&self.fallback, text_id, args)
        {
            return Some(val);
        }
        None
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        Box::new(self.fallbacks.keys())
    }
}
