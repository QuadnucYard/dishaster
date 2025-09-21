use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    Fields, ItemStruct, Result,
    parse::{Parse, ParseStream},
    parse_quote,
};

use crate::utils::{eat_comma, parse_key_value};

pub fn ui_element(attr: TokenStream, item: ItemStruct) -> Result<TokenStream> {
    let info = parse(attr, &item)?;
    create(&info, &item)
}

struct UIElement {
    ident: Ident,
    meta: Meta,
}

fn parse(attr: TokenStream, item: &ItemStruct) -> Result<UIElement> {
    Ok(UIElement {
        ident: item.ident.clone(),
        meta: syn::parse2(attr)?,
    })
}

struct Meta {
    gd: syn::Ident,
    base: Option<syn::Ident>,
}

impl Parse for Meta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let gd = input.parse()?;
        eat_comma(input);
        Ok(Meta {
            gd,
            base: parse_key_value::<kw::base, _>(input)?,
        })
    }
}

fn create(info: &UIElement, item: &ItemStruct) -> Result<TokenStream> {
    let ident = &info.ident;

    let mut item = item.clone();
    let gd = &info.meta.gd;
    if let Fields::Named(fields) = &mut item.fields {
        if let Some(base) = &info.meta.base {
            fields.named.push(parse_quote! { pub base: #base });
        }
        fields.named.push(parse_quote! { gd: godot::obj::Gd<#gd> });
    }

    Ok(quote! {
        #[derive(Clone)]
        #item

        impl UIElement for #ident {
            fn gd(&self) -> godot::obj::Gd<godot::classes::Control> {
                self.gd.clone().upcast()
            }

            fn dup(&self) -> Self {
                Self::new(self.gd.duplicate().unwrap().cast())
            }
        }

        impl From<godot::obj::Gd<#gd>> for #ident {
            fn from(value: godot::obj::Gd<#gd>) -> Self {
                Self::new(value)
            }
        }

        unsafe impl Send for #ident {}
        unsafe impl Sync for #ident {}
    })
}

mod kw {
    syn::custom_keyword!(base);
}
