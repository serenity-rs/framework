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

pub fn default_error_type() -> Box<Type> {
    to_type(quote! {
        serenity_framework::DefaultError
    })
}

pub fn command_type(data: &Type, error: &Type) -> Path {
    to_path(quote! {
        serenity_framework::command::Command<#data, #error>
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

pub fn required_argument_from_str_func() -> Path {
    to_path(quote! {
        serenity_framework::argument::required_argument_from_str
    })
}

pub fn required_argument_parse_func() -> Path {
    to_path(quote! {
        serenity_framework::argument::required_argument_parse
    })
}

pub fn optional_argument_from_str_func() -> Path {
    to_path(quote! {
        serenity_framework::argument::optional_argument_from_str
    })
}

pub fn optional_argument_parse_func() -> Path {
    to_path(quote! {
        serenity_framework::argument::optional_argument_parse
    })
}

pub fn variadic_arguments_from_str_func() -> Path {
    to_path(quote! {
        serenity_framework::argument::variadic_arguments_from_str
    })
}

pub fn variadic_arguments_parse_func() -> Path {
    to_path(quote! {
        serenity_framework::argument::variadic_arguments_parse
    })
}

pub fn rest_argument_from_str_func() -> Path {
    to_path(quote! {
        serenity_framework::argument::rest_argument_from_str
    })
}

pub fn rest_argument_parse_func() -> Path {
    to_path(quote! {
        serenity_framework::argument::rest_argument_parse
    })
}

pub fn check_type(data: &Type, error: &Type) -> Path {
    to_path(quote! {
        serenity_framework::check::Check<#data, #error>
    })
}

pub fn check_builder_type() -> Path {
    to_path(quote! {
        serenity_framework::check::CheckBuilder
    })
}
