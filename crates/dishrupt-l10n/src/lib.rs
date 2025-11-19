//! Localization with Fluent

mod builtins;

#[doc(hidden)]
pub mod private {
    use std::{
        collections::HashMap,
        sync::{Arc, LazyLock, OnceLock, RwLock},
    };

    use fluent::{FluentResource, FluentValue, concurrent::FluentBundle};
    use fluent_templates::ArcLoader;
    use unic_langid::LanguageIdentifier;

    use crate::builtins;

    static LANG: LazyLock<RwLock<LanguageIdentifier>> =
        LazyLock::new(|| unic_langid::langid!("zh-CN").into());

    static LOCALES: OnceLock<ArcLoader> = OnceLock::new();

    fn get_locales() -> &'static ArcLoader {
        LOCALES.get_or_init(|| build_loader("locales/"))
    }

    /// Triggers the initialization of LOCALES with default path.
    pub fn init() {
        get_locales();
    }

    /// Initialize localization with a custom source path.
    /// This is useful for tests that need to load locales from a different directory.
    /// Must be called before any localization operations.
    pub fn init_with_path(path: &str) {
        let loader = build_loader(path);

        if LOCALES.set(loader).is_err() {
            panic!(
                "LOCALES already initialized - init_with_path must be called before any localization operations"
            );
        }
    }

    fn build_loader(path: &str) -> ArcLoader {
        ArcLoader::builder(path, LANG.read().unwrap().clone())
            .customize(|bundle| {
                bundle.set_use_isolating(false);
                add_builtin(bundle);
            })
            .build()
            .expect("build ArcLoader with custom path")
    }

    fn add_builtin(bundle: &mut FluentBundle<Arc<FluentResource>>) {
        // The builtin function in v0.16.1 are missing, so we have to implement them by ourself.
        bundle
            .add_function("NUM", builtins::number)
            .expect("Failed to add function `NUM` to the bundle.");
        bundle
            .add_function("PCT", builtins::percent)
            .expect("Failed to add function `PCT` to the bundle.");
    }

    pub fn tr_impl(id: &str, args: Option<&HashMap<&'static str, FluentValue>>) -> String {
        get_locales()
            .lookup_single_language(&LANG.read().unwrap(), id, args)
            .unwrap_or_else(|err| {
                eprintln!("Failed to get message `{id}`: {err}.");
                id.to_string()
            })
    }

    pub fn tr_with<T>(id: &str, args: Option<&HashMap<T, FluentValue>>) -> String
    where
        T: AsRef<str>,
    {
        get_locales()
            .lookup_single_language(&LANG.read().unwrap(), id, args)
            .unwrap_or_else(|err| {
                eprintln!("Failed to get message `{id}`: {err}.");
                id.to_string()
            })
    }

    /// Translate a message id into message string.
    pub fn try_tr_plain(id: &str) -> Option<String> {
        get_locales()
            .lookup_single_language::<&'static str>(&LANG.read().unwrap(), id, None)
            .ok()
    }
}

// Re-export for convenience
pub use fluent;
pub use private::{init, init_with_path, try_tr_plain};

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
            args.insert($key, $value.into());
        )*
        $crate::private::tr_impl($id, Some(&args))
    }};
    ($fmt: literal $(, $fmt_args: expr)+) => {
        $crate::private::tr_impl(&format!($fmt $(,$fmt_args)+ ), None)
    };
    ($fmt: literal $(, $fmt_args: expr)+ ; $args:expr) => {{
        let args = match &$args { args => args };
        $crate::private::tr_with(&format!($fmt $(,$fmt_args)+ ), Some( &$args ))
    }};
}
