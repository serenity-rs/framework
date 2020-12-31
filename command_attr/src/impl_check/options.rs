use crate::utils::{parse_bool, Attr};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Result};

use std::convert::TryInto;

pub fn parse_options(attrs: &[Attribute]) -> Result<(TokenStream, Vec<Attribute>)> {
    let mut stream = TokenStream::new();
    let mut normal = Vec::new();

    for attr in attrs {
        let name = attr.path.get_ident().unwrap().to_string();

        let tokens = match name.as_str() {
            "check_in_help" => check_in_help(attr.try_into()?)?,
            "display_in_help" => display_in_help(attr.try_into()?)?,
            _ => {
                normal.push(attr.clone());
                continue;
            },
        };

        stream.extend(tokens);
    }

    Ok((stream, normal))
}

fn check_in_help(attr: Attr) -> Result<TokenStream> {
    let check = parse_bool(&attr)?;

    Ok(quote! {
        .check_in_help(#check)
    })
}

fn display_in_help(attr: Attr) -> Result<TokenStream> {
    let display = parse_bool(&attr)?;

    Ok(quote! {
        .display_in_help(#display)
    })
}
