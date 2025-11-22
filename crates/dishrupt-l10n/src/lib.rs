//! Localization with Fluent

mod builtins;

use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, LazyLock, OnceLock, RwLock},
};

use fluent_bundle::{FluentResource, FluentValue, concurrent::FluentBundle};
use fluent_loader::SyncLoader;
use fluent_templates::ArcLoader;
use unic_langid::LanguageIdentifier;

static LANG: LazyLock<RwLock<LanguageIdentifier>> =
    LazyLock::new(|| unic_langid::langid!("zh-CN").into());

static LOCALES: OnceLock<Box<dyn SyncLoader>> = OnceLock::new();

/// Get the current language identifier.
pub fn get_lang() -> LanguageIdentifier {
    LANG.read().unwrap().clone()
}

/// Get the global localization loader.
pub fn get_locales() -> &'static dyn SyncLoader {
    LOCALES
        .get_or_init(|| Box::new(build_arc_loader("locales/")))
        .as_ref()
}

/// Set the global localization loader.
pub fn set_locales(loader: impl SyncLoader + 'static) {
    if LOCALES.set(Box::new(loader)).is_err() {
        panic!(
            "LOCALES already initialized - set_locales must be called before any localization operations"
        );
    }
}

/// Initialize localization with a custom source path.
/// This is useful for tests that need to load locales from a different directory.
/// Must be called before any localization operations.
pub fn init_with_path(path: &str) {
    set_locales(build_arc_loader(path));
}

fn build_arc_loader(path: &str) -> ArcLoader {
    ArcLoader::builder(path, LANG.read().unwrap().clone())
        .customize(|bundle| {
            bundle.set_use_isolating(false);
            add_builtins(bundle);
        })
        .build()
        .expect("build ArcLoader with custom path")
}

/// Add builtin functions to a Fluent bundle.
#[doc(hidden)]
pub fn add_builtins(bundle: &mut FluentBundle<Arc<FluentResource>>) {
    // The builtin function in v0.16.1 are missing, so we have to implement them by ourself.
    bundle
        .add_function("NUM", builtins::number)
        .expect("Failed to add function `NUM` to the bundle.");
    bundle
        .add_function("PCT", builtins::percent)
        .expect("Failed to add function `PCT` to the bundle.");
}

/// Translate a message id into message string.
pub fn try_tr_plain(id: &str) -> Option<String> {
    get_locales().try_lookup_complete(&LANG.read().unwrap(), id, None)
}

#[doc(hidden)]
pub mod private {
    use super::*;

    pub fn tr_impl(id: &str, args: Option<&HashMap<Cow<'static, str>, FluentValue>>) -> String {
        let lang = LANG.read().unwrap();
        get_locales()
            .try_lookup_complete(&lang, id, args)
            .unwrap_or_else(|| format!("Unknown localization key: {id:?}"))
    }

    pub fn tr_impl_generic<T>(id: &str, args: &HashMap<T, FluentValue>) -> String
    where
        T: AsRef<str>,
    {
        tr_impl(
            id,
            Some(
                &args
                    .iter()
                    .map(|(k, v)| (Cow::Owned(k.as_ref().to_string()), v.clone()))
                    .collect::<HashMap<Cow<'static, str>, FluentValue>>(),
            ),
        )
    }
}

// Re-export for convenience
pub use fluent_bundle as fluent;

/// Translate a message id into message string. Returns [`String`].
///
/// Overloads:
///
/// - `tr!(id)`: No args.
/// - `tr!(id, key1 = value1)`: With args key1=value1, ... Keys are string literals.
/// - `tr!(fmt, fmt_args..)`: Format id with fmt_args, with no message args.
/// - `tr!(fmt, fmt_args.. => params)`: Format id with fmt_args, with message args in `params`.
#[macro_export]
macro_rules! tr {
    // ($id: expr, $($key: expr => $value: expr),* $(,)?) => {
    //     tr_impl($id, fluent_args![ $($key => $value,)* ];)
    // };
    ($id: expr) => {
        $crate::private::tr_impl($id, None)
    };
    ($id: expr, $($key: literal = $value: expr),* $(,)?) => {{
        let mut args = std::collections::HashMap::new();
        $(
            args.insert(std::borrow::Cow::from($key), $value.into());
        )*
        $crate::private::tr_impl($id, Some(&args))
    }};
    ($fmt: literal $(, $fmt_args: expr)+) => {
        $crate::private::tr_impl(&format!($fmt $(,$fmt_args)+ ), None)
    };
    ($fmt: literal $(, $fmt_args: expr)+ ; $args:expr) => {{
        let args = match &$args { args => args };
        $crate::private::tr_impl_generic(&format!($fmt $(,$fmt_args)+ ), &$args)
    }};
}
