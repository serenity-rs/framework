use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse2, Attribute, Error, FnArg, GenericArgument, Lit, LitStr, Meta};
use syn::{NestedMeta, Path, PathArguments, Result, Signature, Token, Type};

use std::convert::TryFrom;

pub fn crate_name() -> Ident {
    Ident::new("serenity_framework", Span::call_site())
}

pub fn default_data(crate_name: &Ident) -> Box<Type> {
    parse2(quote!(#crate_name::DefaultData)).unwrap()
}

pub fn default_error(crate_name: &Ident) -> Box<Type> {
    parse2(quote!(#crate_name::DefaultError)).unwrap()
}

pub struct AttributeArgs(pub Vec<String>);

impl Parse for AttributeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut v = Vec::new();

        loop {
            if input.is_empty() {
                break;
            }

            v.push(input.parse::<LitStr>()?.value());

            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        Ok(Self(v))
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Ident(Ident),
    Lit(Lit),
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Value::Ident(ident) => ident.to_tokens(tokens),
            Value::Lit(lit) => lit.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attr {
    pub path: Path,
    pub values: Vec<Value>,
}

impl Attr {
    pub fn new(path: Path, values: Vec<Value>) -> Self {
        Self { path, values }
    }
}

impl ToTokens for Attr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Attr { path, values } = self;

        tokens.extend(if values.is_empty() {
            quote!(#[#path])
        } else {
            quote!(#[#path(#(#values)*,)])
        });
    }
}

impl TryFrom<&Attribute> for Attr {
    type Error = Error;

    fn try_from(attr: &Attribute) -> Result<Self> {
        parse_attribute(attr)
    }
}

pub fn parse_attribute(attr: &Attribute) -> Result<Attr> {
    let meta = attr.parse_meta()?;

    match meta {
        Meta::Path(p) => Ok(Attr::new(p, Vec::new())),
        Meta::List(l) => {
            let path = l.path;
            let values = l
                .nested
                .into_iter()
                .map(|m| match m {
                    NestedMeta::Lit(lit) => Ok(Value::Lit(lit)),
                    NestedMeta::Meta(m) => match m {
                        Meta::Path(p) => Ok(Value::Ident(p.get_ident().unwrap().clone())),
                        _ => Err(Error::new(
                            m.span(),
                            "nested lists or name values are not supported",
                        )),
                    },
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(Attr::new(path, values))
        },
        Meta::NameValue(nv) => Ok(Attr::new(nv.path, vec![Value::Lit(nv.lit)])),
    }
}

pub fn parse_identifiers(attr: &Attr) -> Result<Vec<Ident>> {
    attr.values
        .iter()
        .map(|v| match v {
            Value::Ident(ident) => Ok(ident.clone()),
            Value::Lit(lit) => Err(Error::new(lit.span(), "literals are forbidden")),
        })
        .collect::<Result<Vec<_>>>()
}

pub fn parse_value<T>(attr: &Attr, f: impl FnOnce(&Value) -> Result<T>) -> Result<T> {
    if attr.values.is_empty() {
        return Err(Error::new(attr.span(), "attribute input must not be empty"));
    }

    if attr.values.len() > 1 {
        return Err(Error::new(
            attr.span(),
            "attribute input must not exceed more than one argument",
        ));
    }

    f(&attr.values[0])
}

pub fn parse_identifier(attr: &Attr) -> Result<Ident> {
    parse_value(attr, |value| {
        Ok(match value {
            Value::Ident(ident) => ident.clone(),
            _ => return Err(Error::new(value.span(), "argument must be an identifier")),
        })
    })
}

pub fn parse_string(attr: &Attr) -> Result<String> {
    parse_value(attr, |value| {
        Ok(match value {
            Value::Lit(Lit::Str(s)) => s.value(),
            _ => return Err(Error::new(value.span(), "argument must be a string")),
        })
    })
}

pub fn parse_bool(attr: &Attr) -> Result<bool> {
    parse_value(attr, |value| {
        Ok(match value {
            Value::Lit(Lit::Bool(b)) => b.value,
            _ => return Err(Error::new(value.span(), "argument must be a boolean")),
        })
    })
}

pub fn parse_generics(
    sig: &Signature,
    default_data: Box<Type>,
    default_error: Box<Type>,
) -> Result<(Box<Type>, Box<Type>)> {
    let ctx = get_first_parameter(sig)?;
    let ty = get_type(ctx)?;
    let path = get_path(ty)?;
    let mut arguments = get_generic_arguments(path)?;

    let data = match arguments.next() {
        Some(GenericArgument::Lifetime(_)) => match arguments.next() {
            Some(arg) => get_generic_type(arg)?,
            None => default_data,
        },
        Some(arg) => get_generic_type(arg)?,
        None => default_data,
    };

    let error = match arguments.next() {
        Some(arg) => get_generic_type(arg)?,
        None => default_error,
    };

    Ok((data, error))
}

fn get_first_parameter(sig: &Signature) -> Result<&FnArg> {
    match sig.inputs.first() {
        Some(arg) => Ok(arg),
        None => Err(Error::new(
            sig.inputs.span(),
            "the function must have parameters",
        )),
    }
}

fn get_type(arg: &FnArg) -> Result<&Type> {
    match arg {
        FnArg::Typed(t) => Ok(&*t.ty),
        _ => Err(Error::new(
            arg.span(),
            "`self` cannot be used as the context type",
        )),
    }
}

fn get_path(t: &Type) -> Result<&Path> {
    match t {
        Type::Path(p) => Ok(&p.path),
        _ => Err(Error::new(
            t.span(),
            "first parameter must be a path to a context type",
        )),
    }
}

fn get_generic_arguments(path: &Path) -> Result<impl Iterator<Item = &GenericArgument> + '_> {
    match &path.segments.last().unwrap().arguments {
        PathArguments::None => Ok(Vec::new().into_iter()),
        PathArguments::AngleBracketed(arguments) =>
            Ok(arguments.args.iter().collect::<Vec<_>>().into_iter()),
        _ => Err(Error::new(
            path.span(),
            "context type cannot have generic parameters in parenthesis",
        )),
    }
}

fn get_generic_type(arg: &GenericArgument) -> Result<Box<Type>> {
    match arg {
        GenericArgument::Type(t) => Ok(Box::new(t.clone())),
        _ => Err(Error::new(arg.span(), "generic parameter must be a type")),
    }
}
