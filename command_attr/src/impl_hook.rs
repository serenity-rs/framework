use proc_macro2::{Span, TokenStream};

use syn::parse2;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Error, FnArg, GenericParam, Generics, ItemFn, Lifetime};
use syn::{LifetimeDef, Result, ReturnType, Signature, Token, Type};

use quote::quote;

pub fn impl_hook(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    if !attr.is_empty() {
        return Err(Error::new(
            attr.span(),
            "parameters to the `#[hook]` macro are ignored",
        ));
    }

    let fun = parse2::<ItemFn>(input)?;

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
        mut generics,
        ..
    } = sig;

    if asyncness.is_none() {
        return Err(Error::new(sig_span, "`async` keyword is missing"));
    }

    let output = match output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, t) => quote!(#t),
    };

    add_fut_lifetime(&mut generics);
    populate_lifetime(&mut inputs);

    let result = quote! {
        #(#attrs)*
        #vis fn #ident #generics (#inputs) -> std::pin::Pin<Box<dyn std::future::Future<Output = #output> + 'fut + Send>> {
            Box::pin(async move {
                // Nudge the compiler into providing us with a good error message
                // when the return type of the body does not match with the return
                // type of the function.
                let result: #output = #block;
                result
            })
        }
    };

    Ok(result)
}

fn add_fut_lifetime(generics: &mut Generics) {
    generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeDef {
            attrs: Vec::default(),
            lifetime: Lifetime::new("'fut", Span::call_site()),
            colon_token: None,
            bounds: Punctuated::default(),
        }),
    );
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
