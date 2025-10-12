//! Localization with Fluent

mod builtins;

#[doc(hidden)]
pub mod private {
    use std::{
        collections::HashMap,
        sync::{Arc, LazyLock, Mutex},
    };

    use fluent::{FluentResource, FluentValue, concurrent::FluentBundle};
    use fluent_templates::ArcLoader;
    use unic_langid::LanguageIdentifier;

    use crate::builtins;

    static LOCALES: LazyLock<ArcLoader> = LazyLock::new(|| {
        ArcLoader::builder("locales/", unic_langid::langid!("zh-CN"))
            // .shared_resources(Some(&["locales/core.ftl".into()]))
            .customize(|bundle| {
                bundle.set_use_isolating(false);
                add_builtin(bundle);
            })
            .build()
            .expect("build ArcLoader")
    });

    static LANG: LazyLock<Mutex<LanguageIdentifier>> =
        LazyLock::new(|| unic_langid::langid!("zh-CN").into());

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
        LOCALES
            .lookup_single_language(&LANG.lock().unwrap(), id, args)
            .unwrap_or_else(|err| {
                eprintln!("Failed to get message `{id}`: {err}.");
                id.to_string()
            })
    }

    /// Translate a message id into message string.
    pub fn try_tr_plain(id: &str) -> Option<String> {
        LOCALES
            .lookup_single_language::<&'static str>(&LANG.lock().unwrap(), id, None)
            .ok()
    }
}

pub use private::try_tr_plain;

/// Translate a message id into message string. Returns [`String`].
///
/// Overloads:
///
/// - `tr!(id)`: No args.
/// - `tr!(id, key1 = value1)`: With args key1=value1, ... Keys are string literals.
/// - `tr!(fmt, fmt_args..)`: Format id with fmt_args, with no message args.
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
    ($fmt: literal $(, $args: expr)+) => {
        $crate::private::tr_impl(&format!($fmt $(,$args)+ ), None)
    };
}
