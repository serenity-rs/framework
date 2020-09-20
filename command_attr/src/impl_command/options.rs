use crate::utils::{parse_bool, parse_identifier, parse_identifiers, parse_string, Attr};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Result};

pub fn parse_options(builtin: &[Attr]) -> Result<TokenStream2> {
    let mut stream = TokenStream2::new();

    for attr in builtin {
        let name = attr.path.get_ident().unwrap().to_string();

        match name.as_str() {
            "subcommand" => stream.extend(subcommand(attr)?),
            "subcommands" => stream.extend(subcommands(attr)?),
            "description" => stream.extend(description(attr)?),
            "dynamic_description" => stream.extend(dynamic_description(attr)?),
            "usage" => stream.extend(usage(attr)?),
            "dynamic_usage" => stream.extend(dynamic_usage(attr)?),
            "example" => stream.extend(example(attr)?),
            "dynamic_examples" => stream.extend(dynamic_examples(attr)?),
            "help_available" => stream.extend(help_available(attr)?),
            "check" => stream.extend(check(attr)?),
            _ => return Err(Error::new(attr.path.span(), "invalid attribute")),
        }
    }

    Ok(stream)
}

fn subcommand(attr: &Attr) -> Result<TokenStream2> {
    let subcommand = parse_identifier(attr)?;

    Ok(quote! {
        .subcommand(#subcommand)
    })
}

fn subcommands(attr: &Attr) -> Result<TokenStream2> {
    let subcommands = parse_identifiers(attr)?;

    Ok(quote! {
        #(.subcommand(#subcommands))*
    })
}

fn description(attr: &Attr) -> Result<TokenStream2> {
    let desc = parse_string(attr)?;

    Ok(quote! {
        .description(#desc)
    })
}

fn dynamic_description(attr: &Attr) -> Result<TokenStream2> {
    let desc = parse_identifier(attr)?;

    Ok(quote! {
        .dynamic_description(#desc)
    })
}

fn usage(attr: &Attr) -> Result<TokenStream2> {
    let usage = parse_string(attr)?;

    Ok(quote! {
        .usage(#usage)
    })
}

fn dynamic_usage(attr: &Attr) -> Result<TokenStream2> {
    let usage = parse_identifier(attr)?;

    Ok(quote! {
        .dynamic_usage(#usage)
    })
}

fn example(attr: &Attr) -> Result<TokenStream2> {
    let example = parse_string(attr)?;

    Ok(quote! {
        .example(#example)
    })
}

fn dynamic_examples(attr: &Attr) -> Result<TokenStream2> {
    let examples = parse_identifier(attr)?;

    Ok(quote! {
        .dynamic_examples(#examples)
    })
}

fn help_available(attr: &Attr) -> Result<TokenStream2> {
    let help = parse_bool(attr)?;

    Ok(quote! {
        .help_available(#help)
    })
}

fn check(attr: &Attr) -> Result<TokenStream2> {
    let check = parse_identifier(attr)?;

    Ok(quote! {
        .check(#check)
    })
}
