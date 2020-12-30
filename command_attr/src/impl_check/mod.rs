use crate::context::{check_builder_type, check_fn, check_type, Context};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse2;
use syn::{ItemFn, Result};

mod options;

pub fn impl_check(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let mut fun = parse2::<ItemFn>(input)?;

    let name = if attr.is_empty() {
        fun.sig.ident.to_string()
    } else {
        parse2::<syn::LitStr>(attr)?.value()
    };

    let ctx = Context::new(&fun)?;

    let builder_fn = builder_fn(&ctx, &mut fun, &name)?;
    let check_fn = check_fn(&ctx, &fun);

    let result = quote! {
        #builder_fn

        #check_fn
    };

    Ok(result)
}

fn builder_fn(ctx: &Context, function: &mut ItemFn, name: &str) -> Result<TokenStream> {
    // Derive the name of the builder from the check function.
    // Prepend the check function's name with an underscore to avoid name
    // collisions.
    let builder_name = function.sig.ident.clone();
    let function_name = format_ident!("_{}", builder_name);
    function.sig.ident = function_name.clone();

    let check_builder = check_builder_type(ctx);
    let check = check_type(ctx);

    let vis = function.vis.clone();
    let (builtin, external) = options::parse_options(&function.attrs)?;
    function.attrs = external.clone();

    Ok(quote! {
        #(#external)*
        #vis fn #builder_name() -> #check {
            #check_builder::new(#name)
                .function(#function_name)
                #builtin
                .build()
        }
    })
}
