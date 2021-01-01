use crate::utils;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ItemFn, Result, Type};

pub struct Context {
    pub crate_name: Ident,
    pub ctx_name: Ident,
    pub data: Box<Type>,
    pub error: Box<Type>,
}

impl Context {
    pub fn new(function: &ItemFn) -> Result<Self> {
        let crate_name = utils::crate_name();
        let default_data = utils::default_data(&crate_name);
        let default_error = utils::default_error(&crate_name);

        let (ctx_name, data, error) =
            utils::parse_generics(&function.sig, default_data, default_error)?;

        Ok(Self {
            crate_name,
            ctx_name,
            data,
            error,
        })
    }
}

pub fn command_type(ctx: &Context) -> TokenStream {
    let Context {
        crate_name,
        data,
        error,
        ..
    } = ctx;

    quote! {
        #crate_name::command::Command<#data, #error>
    }
}

pub fn command_builder_type(ctx: &Context) -> TokenStream {
    let crate_name = &ctx.crate_name;

    quote! {
        #crate_name::command::CommandBuilder
    }
}

pub fn command_fn(ctx: &Context, function: &ItemFn) -> TokenStream {
    let crate_name = &ctx.crate_name;

    quote! {
        #[#crate_name::prelude::hook]
        #[doc(hidden)]
        #function
    }
}

pub fn req_argument_func(ctx: &Context) -> TokenStream {
    let crate_name = &ctx.crate_name;

    quote! {
        #crate_name::argument::req_argument
    }
}

pub fn opt_argument_func(ctx: &Context) -> TokenStream {
    let crate_name = &ctx.crate_name;

    quote! {
        #crate_name::argument::opt_argument
    }
}

pub fn var_arguments_func(ctx: &Context) -> TokenStream {
    let crate_name = &ctx.crate_name;

    quote! {
        #crate_name::argument::var_arguments
    }
}

pub fn argument_segments_type(ctx: &Context) -> TokenStream {
    let crate_name = &ctx.crate_name;

    quote! {
        #crate_name::utils::ArgumentSegments
    }
}

pub fn check_type(ctx: &Context) -> TokenStream {
    let Context {
        crate_name,
        data,
        error,
        ..
    } = ctx;

    quote! {
        #crate_name::check::Check<#data, #error>
    }
}

pub fn check_builder_type(ctx: &Context) -> TokenStream {
    let crate_name = &ctx.crate_name;

    quote! {
        #crate_name::check::CheckBuilder
    }
}

pub fn check_fn(ctx: &Context, function: &ItemFn) -> TokenStream {
    let crate_name = &ctx.crate_name;

    quote! {
        #[#crate_name::prelude::hook]
        #[doc(hidden)]
        #function
    }
}
