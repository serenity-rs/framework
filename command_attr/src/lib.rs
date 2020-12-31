use proc_macro::TokenStream;

mod context;
mod utils;

mod impl_check;
mod impl_command;
mod impl_hook;

use impl_check::impl_check;
use impl_command::impl_command;
use impl_hook::impl_hook;

#[proc_macro_attribute]
pub fn command(attr: TokenStream, input: TokenStream) -> TokenStream {
    match impl_command(attr.into(), input.into()) {
        Ok(stream) => stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn check(attr: TokenStream, input: TokenStream) -> TokenStream {
    match impl_check(attr.into(), input.into()) {
        Ok(stream) => stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn hook(attr: TokenStream, input: TokenStream) -> TokenStream {
    match impl_hook(attr.into(), input.into()) {
        Ok(stream) => stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
