use crate::utils::{parse_bool, parse_identifier, parse_identifiers, parse_string, Attr};

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
            "subcommands" => subcommands(attr.try_into()?)?,
            "doc" | "description" => description(attr.try_into()?)?,
            "dynamic_description" => dynamic_description(attr.try_into()?)?,
            "usage" => usage(attr.try_into()?)?,
            "dynamic_usage" => dynamic_usage(attr.try_into()?)?,
            "example" => example(attr.try_into()?)?,
            "dynamic_examples" => dynamic_examples(attr.try_into()?)?,
            "help_available" => help_available(attr.try_into()?)?,
            "check" => check(attr.try_into()?)?,
            _ => {
                normal.push(attr.clone());
                continue;
            },
        };

        stream.extend(tokens);
    }

    Ok((stream, normal))
}

fn subcommands(attr: Attr) -> Result<TokenStream> {
    let subcommands = parse_identifiers(&attr)?;

    Ok(quote! {
        #(.subcommand(#subcommands))*
    })
}

fn description(attr: Attr) -> Result<TokenStream> {
    let mut desc = parse_string(&attr)?;

    if desc.starts_with(' ') {
        desc.remove(0);
    }

    Ok(quote! {
        .description(#desc)
    })
}

fn dynamic_description(attr: Attr) -> Result<TokenStream> {
    let desc = parse_identifier(&attr)?;

    Ok(quote! {
        .dynamic_description(#desc)
    })
}

fn usage(attr: Attr) -> Result<TokenStream> {
    let usage = parse_string(&attr)?;

    Ok(quote! {
        .usage(#usage)
    })
}

fn dynamic_usage(attr: Attr) -> Result<TokenStream> {
    let usage = parse_identifier(&attr)?;

    Ok(quote! {
        .dynamic_usage(#usage)
    })
}

fn example(attr: Attr) -> Result<TokenStream> {
    let example = parse_string(&attr)?;

    Ok(quote! {
        .example(#example)
    })
}

fn dynamic_examples(attr: Attr) -> Result<TokenStream> {
    let examples = parse_identifier(&attr)?;

    Ok(quote! {
        .dynamic_examples(#examples)
    })
}

fn help_available(attr: Attr) -> Result<TokenStream> {
    let help = parse_bool(&attr)?;

    Ok(quote! {
        .help_available(#help)
    })
}

fn check(attr: Attr) -> Result<TokenStream> {
    let check = parse_identifier(&attr)?;

    Ok(quote! {
        .check(#check)
    })
}
