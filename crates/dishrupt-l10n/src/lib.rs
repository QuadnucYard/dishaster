//! Localization with Fluent

mod builtins;
mod service;

use std::{borrow::Cow, collections::HashMap, sync::LazyLock};

use fluent_bundle::FluentValue;
pub use unic_langid::langid;

pub use self::service::{L10nService, build_arc_loader, default_customizer};

/// The global localization service instance.
pub static L10N_SERVICE: LazyLock<L10nService> =
    LazyLock::new(|| L10nService::new_with_lang(langid!("zh-CN")));

/// Translate a message id into message string.
pub fn try_tr_plain(id: &str) -> Option<String> {
    let lang = L10N_SERVICE.get_lang();
    L10N_SERVICE
        .get_locales()
        .try_lookup_complete(&lang, id, None)
}

#[doc(hidden)]
pub mod private {
    use super::*;

    pub fn tr_impl(id: &str, args: Option<&HashMap<Cow<'static, str>, FluentValue>>) -> String {
        let lang = L10N_SERVICE.get_lang();
        L10N_SERVICE
            .get_locales()
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
