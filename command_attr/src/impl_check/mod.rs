use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse2;
use syn::{ItemFn, Result, Type};

use crate::paths;
use crate::utils;

mod options;

use options::Options;

pub fn impl_check(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let mut fun = parse2::<ItemFn>(input)?;

    let name = if attr.is_empty() {
        fun.sig.ident.to_string()
    } else {
        parse2::<syn::LitStr>(attr)?.value()
    };

    let (_, data, error) = utils::parse_generics(&fun.sig)?;
    let options = Options::parse(&mut fun.attrs)?;

    let builder_fn = builder_fn(&data, &error, &mut fun, &name, &options);

    let hook_macro = paths::hook_macro();

    let result = quote! {
        #builder_fn

        #[#hook_macro]
        #[doc(hidden)]
        #fun
    };

    Ok(result)
}

fn builder_fn(
    data: &Type,
    error: &Type,
    function: &mut ItemFn,
    name: &str,
    options: &Options,
) -> TokenStream {
    // Derive the name of the builder from the check function.
    // Prepend the check function's name with an underscore to avoid name
    // collisions.
    let builder_name = function.sig.ident.clone();
    let function_name = format_ident!("_{}", builder_name);
    function.sig.ident = function_name.clone();

    let check_builder = paths::check_builder_type();
    let check = paths::check_type(data, error);

    let vis = &function.vis;
    let external = &function.attrs;

    quote! {
        #(#external)*
        #vis fn #builder_name() -> #check {
            #check_builder::new(#name)
                .function(#function_name)
                #options
                .build()
        }
    }
}
