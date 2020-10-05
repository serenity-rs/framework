use proc_macro::TokenStream;

use proc_macro2::{Span, TokenStream as TokenStream2};

use syn::parse;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Error, FnArg, ItemFn, Lifetime, Result, ReturnType, Signature, Token, Type};

use quote::quote;

pub fn impl_hook(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    if !attr.is_empty() {
        let attr = TokenStream2::from(attr);
        return Err(Error::new(
            attr.span(),
            "parameters to the `#[hook]` macro are ignored",
        ));
    }

    let fun = parse::<ItemFn>(input)?;

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = fun;

    let sig_span = sig.span();
    let Signature {
        asyncness,
        ident,
        mut inputs,
        output,
        ..
    } = sig;

    if asyncness.is_none() {
        return Err(Error::new(sig_span, "`async` keyword is missing"));
    }

    let output = match output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, t) => quote!(#t),
    };

    populate_lifetime(&mut inputs);

    let result = quote! {
        #(#attrs)*
        #vis fn #ident<'fut>(#inputs) -> std::pin::Pin<Box<dyn std::future::Future<Output = #output> + 'fut + Send>> {
            Box::pin(async move {
                // Nudge the compiler into providing us with a good error message
                // when the return type of the body does not match with the return
                // type of the function.
                let result: #output = #block;
                result
            })
        }
    };

    Ok(result.into())
}

fn populate_lifetime(inputs: &mut Punctuated<FnArg, Token![,]>) {
    for input in inputs {
        if let FnArg::Typed(kind) = input {
            if let Type::Reference(ty) = &mut *kind.ty {
                ty.lifetime = Some(Lifetime::new("'fut", Span::call_site()));
            }
        }
    }
}
