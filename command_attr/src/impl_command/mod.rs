use crate::utils::{self, AttributeArgs};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::parse2;
use syn::{ItemFn, Result, Type};

mod options;

pub fn impl_command(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let mut fun = parse2::<ItemFn>(input)?;

    let names = if attr.is_empty() {
        vec![fun.sig.ident.to_string()]
    } else {
        parse2::<AttributeArgs>(attr)?.0
    };

    let ctx = Context::new(&fun)?;

    let builder_fn = builder_fn(&ctx, &mut fun, names)?;
    let command_fn = command_fn(&ctx, &fun);

    let result = quote! {
        #builder_fn

        #command_fn
    };

    Ok(result)
}

struct Context {
    crate_name: Ident,
    data: Box<Type>,
    error: Box<Type>,
}

impl Context {
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
}

fn command_type(ctx: &Context) -> TokenStream {
    let Context {
        crate_name,
        data,
        error,
    } = ctx;

    quote! {
        #crate_name::command::Command<#data, #error>
    }
}

fn command_builder_type(ctx: &Context) -> TokenStream {
    let crate_name = &ctx.crate_name;

    quote! {
        #crate_name::command::CommandBuilder
    }
}

fn command_fn(ctx: &Context, function: &ItemFn) -> TokenStream {
    let crate_name = &ctx.crate_name;

    quote! {
        #[#crate_name::prelude::hook]
        #[doc(hidden)]
        #function
    }
}

fn builder_fn(ctx: &Context, function: &mut ItemFn, mut names: Vec<String>) -> Result<TokenStream> {
    let name = names.remove(0);
    let aliases = names;

    // Derive the name of the builder from the command function.
    // Prepend the command function's name with an underscore to avoid name
    // collisions.
    let builder_name = function.sig.ident.clone();
    let function_name = format_ident!("_{}", builder_name);
    function.sig.ident = function_name.clone();

    let command_builder = command_builder_type(ctx);
    let command = command_type(ctx);

    let vis = function.vis.clone();
    let (builtin, external) = options::parse_options(&function.attrs)?;
    function.attrs = external.clone();

    Ok(quote! {
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
