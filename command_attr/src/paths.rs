use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Path, Type};

fn to_path(tokens: TokenStream) -> Path {
    parse2(tokens).unwrap()
}

fn to_type(tokens: TokenStream) -> Box<Type> {
    parse2(tokens).unwrap()
}

pub fn default_data_type() -> Box<Type> {
    to_type(quote! {
        serenity_framework::DefaultData
    })
}

pub fn command_type(data: &Type) -> Path {
    to_path(quote! {
        serenity_framework::command::Command<#data>
    })
}

pub fn command_builder_type() -> Path {
    to_path(quote! {
        serenity_framework::command::CommandBuilder
    })
}

pub fn hook_macro() -> Path {
    to_path(quote! {
        serenity_framework::prelude::hook
    })
}

pub fn argument_segments_type() -> Path {
    to_path(quote! {
        serenity_framework::utils::ArgumentSegments
    })
}

pub fn required_argument_func() -> Path {
    to_path(quote! {
        serenity_framework::argument::required_argument
    })
}

pub fn optional_argument_func() -> Path {
    to_path(quote! {
        serenity_framework::argument::optional_argument
    })
}

pub fn variadic_arguments_func() -> Path {
    to_path(quote! {
        serenity_framework::argument::variadic_arguments
    })
}

pub fn check_type(data: &Type) -> Path {
    to_path(quote! {
        serenity_framework::check::Check<#data>
    })
}

pub fn check_builder_type() -> Path {
    to_path(quote! {
        serenity_framework::check::CheckBuilder
    })
}
