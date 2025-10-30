//! Adapted from typst-macros
#![allow(unused)]

use syn::{
    Result, Token,
    parse::{Parse, ParseStream},
    token::Token,
};

/// A generic parsable array.
struct Array<T>(Vec<T>);

impl<T: Parse> Parse for Array<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        syn::bracketed!(content in input);

        let mut elems = Vec::new();
        while !content.is_empty() {
            let first: T = content.parse()?;
            elems.push(first);
            if !content.is_empty() {
                let _: Token![,] = content.parse()?;
            }
        }

        Ok(Self(elems))
    }
}

/// Parse a metadata key-value pair, separated by `=`.
pub fn parse_key_value<K: Token + Default + Parse, V: Parse>(
    input: ParseStream,
) -> Result<Option<V>> {
    if !input.peek(|_| K::default()) {
        return Ok(None);
    }

    let _: K = input.parse()?;
    let _: Token![=] = input.parse()?;
    let value: V = input.parse::<V>()?;
    eat_comma(input);
    Ok(Some(value))
}

/// Parse a metadata key-array pair, separated by `=`.
pub fn parse_key_value_array<K: Token + Default + Parse, V: Parse>(
    input: ParseStream,
) -> Result<Vec<V>> {
    Ok(parse_key_value::<K, Array<V>>(input)?.map_or(vec![], |array| array.0))
}

/// Parse a metadata flag that can be present or not.
pub fn parse_flag<K: Token + Default + Parse>(input: ParseStream) -> Result<bool> {
    if input.peek(|_| K::default()) {
        let _: K = input.parse()?;
        eat_comma(input);
        return Ok(true);
    }
    Ok(false)
}

/// Parse a comma if there is one.
pub fn eat_comma(input: ParseStream) {
    if input.peek(Token![,]) {
        let _: Token![,] = input.parse().unwrap();
    }
}
