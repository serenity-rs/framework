use crate::utils;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ItemFn, Result, Type};

pub struct Context {
    crate_name: Ident,
    data: Box<Type>,
    error: Box<Type>,
}

impl Context {
    pub fn new(function: &ItemFn) -> Result<Self> {
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

pub fn command_type(ctx: &Context) -> TokenStream {
    let Context {
        crate_name,
        data,
        error,
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

pub fn check_type(ctx: &Context) -> TokenStream {
    let Context {
        crate_name,
        data,
        error,
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
