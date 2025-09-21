mod derive_ui_tree;
mod ui_element;
mod ui_tree;
mod utils;

use proc_macro::TokenStream;
use syn::{self, parse_macro_input};

/// Decorates a struct representing a UI element.
/// It gives the struct `#[derive(Clone)]` attribute and default trait implementation.
///
/// ## Example
///
/// ```ignore
/// #[ui_element]
/// pub struct ControlA {}
/// ```
#[proc_macro_attribute]
pub fn ui_element(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemStruct);
    ui_element::ui_element(attr.into(), item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Decorates a struct representing a UI tree.
/// It adds a `root` field to the struct.
///
/// ## Example
///
/// ```ignore
/// #[ui_tree]
/// pub struct GameUI {}
/// ```
#[proc_macro_attribute]
pub fn ui_tree(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemStruct);
    ui_tree::ui_tree(attr.into(), item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Decorates a UITree impl.
/// It adds a `root` getter to the struct.
///
/// ## Example
///
/// ```ignore
/// #[ui_tree_api]
/// impl UITree for GameUI {}
/// ```
#[proc_macro_attribute]
pub fn ui_tree_api(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemImpl);
    ui_tree::ui_tree_api(attr.into(), item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Build UI trees quickly.
#[proc_macro_derive(UITree, attributes(node, child, child_ui, subtree, init, new))]
pub fn derive_ui_tree(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemStruct);
    derive_ui_tree::derive_ui_tree(item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
