use std::ops::Not;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl, ItemStruct, Result, parse_quote};

pub fn ui_tree(_attr: TokenStream, item: ItemStruct) -> Result<TokenStream> {
    let item = add_root_field(item);
    Ok(quote! {
        #item
    })
}

fn add_root_field(item: ItemStruct) -> ItemStruct {
    match item.fields.to_owned() {
        syn::Fields::Named(mut fields_named) => {
            if item
                .fields
                .iter()
                .any(|f| *f.ident.as_ref().unwrap() == "root")
                .not()
            {
                let mut new_item = item.clone();
                fields_named.named.push(parse_quote! {
                    #[allow(unused_variables)]
                    root: dishrupt_godot_ui::UINode
                });
                new_item.fields = syn::Fields::Named(fields_named);
                new_item
            } else {
                item
            }
        }
        _ => item,
    }
}

pub fn ui_tree_api(_attr: TokenStream, item: ItemImpl) -> Result<TokenStream> {
    let item = add_root_fn(item);
    Ok(quote! {
        #item
    })
}

fn add_root_fn(mut item: ItemImpl) -> ItemImpl {
    let has_root = item.items.iter().any(|it| {
        if let ImplItem::Fn(f) = it {
            f.sig.ident == ""
        } else {
            false
        }
    });
    if has_root {
        return item;
    }
    item.items.push(parse_quote! {
        fn root(&self) -> &UINode { &self.root }
    });
    item.items.push(parse_quote! {
        fn root_mut(&mut self) -> &mut UINode { &mut self.root }
    });
    item
}
