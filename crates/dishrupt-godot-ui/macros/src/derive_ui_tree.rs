use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Error, Expr, Field, Fields, ItemStruct, LitStr, Result, punctuated::Punctuated,
    spanned::Spanned, token::Comma,
};

pub fn derive_ui_tree(item: ItemStruct) -> Result<TokenStream> {
    let name = &item.ident;

    let fields = match item.fields {
        Fields::Named(ref fields_named) => &fields_named.named,
        _ => {
            return Err(Error::new(
                item.span(),
                "The struct is expected to hold named fields",
            ));
        }
    };

    // Get field names and their default values
    let mut has_root_decl = false;
    let mut field_names: Vec<_> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();
    let mut default_inits: Vec<_> = fields
        .iter()
        .map(|f| {
            if f.ident.as_ref().unwrap() == "root" {
                has_root_decl = true;
                quote! { root }
            } else {
                get_default_value(f).unwrap()
            }
        })
        .collect();
    if !has_root_decl {
        field_names.push(format_ident!("root"));
        default_inits.push(quote! { root });
    }

    Ok(quote! {
        impl #name {
            pub fn new(root: dishrupt_godot_ui::UINode) -> Self {
                Self {
                    #(#field_names: #default_inits),*
                }
            }
        }

        impl From<dishrupt_godot_ui::UINode> for #name {
            fn from(value: dishrupt_godot_ui::UINode) -> Self {
                Self::new(value)
            }
        }
    })
}

fn get_default_value(field: &Field) -> Result<TokenStream> {
    for attr in &field.attrs {
        if attr.path().is_ident("init") {
            let tt: Expr = attr.parse_args()?;
            return Ok(quote! { #tt });
        } else if attr.path().is_ident("new") {
            let f = <Punctuated<Expr, Comma>>::parse_terminated;
            let tt = attr.parse_args_with(f)?;
            let ty = &field.ty;
            return Ok(quote! { <#ty>::new(#tt) });
        } else if attr.path().is_ident("node") {
            let path: LitStr = attr.parse_args()?;
            return Ok(quote! { root.child(#path) });
        } else if attr.path().is_ident("child") {
            let path: LitStr = attr.parse_args()?;
            let ty = &field.ty;
            return Ok(quote! { #ty::new(root.child(#path)) });
        } else if attr.path().is_ident("child_ui") {
            let path: LitStr = attr.parse_args()?;
            return Ok(quote! { root.child_ui(#path) });
        } else if attr.path().is_ident("subtree") {
            let path: LitStr = attr.parse_args()?;
            let ty = &field.ty;
            return Ok(quote! { #ty::new(root.child_ui(#path)) });
        }
    }
    Ok(quote! { Default::default() })
}
