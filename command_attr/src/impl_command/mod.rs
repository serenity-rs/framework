use crate::utils::{self, Attr, AttributeArgs};

use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse;
use syn::{Attribute, ItemFn, Result, Type};

mod options;

pub fn impl_command(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let mut fun = parse::<ItemFn>(input)?;

    let names = if attr.is_empty() {
        vec![fun.sig.ident.to_string()]
    } else {
        parse::<AttributeArgs>(attr)?.0
    };

    let (builtin, docs) = parse_attributes(&mut fun)?;

    let common = Common::new(&fun)?;

    let builder_fn = common.builder_fn(&mut fun, names, builtin, docs)?;
    let command_fn = common.command_fn(&fun);

    let result = quote! {
        #builder_fn

        #command_fn
    };

    Ok(result.into())
}

fn parse_attributes(function: &mut ItemFn) -> Result<(Vec<Attr>, Vec<Attribute>)> {
    const COMMAND_ATTRIBUTES: &[&str] = &[
        "subcommand",
        "subcommands",
        "description",
        "dynamic_description",
        "usage",
        "dynamic_usage",
        "examples",
        "dynamic_examples",
        "help_available",
        "check",
    ];

    let (docs, rest) = utils::filter_attributes(&function.attrs, &["doc"]);
    let (builtin, external) = utils::filter_attributes(&rest, COMMAND_ATTRIBUTES);
    let attributes = utils::parse_attributes(&builtin)?;

    function.attrs = external;

    Ok((attributes, docs))
}

struct Common {
    crate_name: Ident,
    data: Box<Type>,
    error: Box<Type>,
}

impl Common {
    fn new(function: &ItemFn) -> Result<Self> {
        let crate_name = utils::crate_name();
        let default_data = utils::default_data(&crate_name);
        let default_error = utils::default_error(&crate_name);

        let (data, error) = utils::parse_generics(&function.sig, default_data, default_error)?;

        Ok(Self {
            crate_name,
            data,
            error,
        })
    }

    fn command_type(&self) -> TokenStream2 {
        let Common {
            crate_name,
            data,
            error,
        } = self;

        quote! {
            #crate_name::command::Command<#data, #error>
        }
    }

    fn command_builder_type(&self) -> TokenStream2 {
        let crate_name = &self.crate_name;

        quote! {
            #crate_name::command::CommandBuilder
        }
    }

    fn command_fn(&self, function: &ItemFn) -> TokenStream2 {
        let crate_name = &self.crate_name;

        quote! {
            #[#crate_name::prelude::hook]
            #[doc(hidden)]
            #function
        }
    }

    fn builder_fn(
        &self,
        function: &mut ItemFn,
        mut names: Vec<String>,
        builtin: Vec<Attr>,
        docs: Vec<Attribute>,
    ) -> Result<TokenStream2> {
        let name = names.remove(0);
        let aliases = names;

        // Derive the name of the builder from the command function.
        // Prepend the command function's name with an underscore to avoid name
        // collisions.
        let builder_name = function.sig.ident.clone();
        let function_name = format_ident!("_{}", builder_name);
        function.sig.ident = function_name.clone();

        let command_builder = self.command_builder_type();
        let command = self.command_type();

        let vis = function.vis.clone();
        let external = function.attrs.clone();
        let builtin = options::parse_options(&builtin)?;

        Ok(quote! {
            #(#docs)*
            #(#external)*
            #vis fn #builder_name() -> #command {
                #command_builder::new(#name)
                    #(.name(#aliases))*
                    .function(#function_name)
                    #builtin
                    .build()
            }
        })
    }
}
