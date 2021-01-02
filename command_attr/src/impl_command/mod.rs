use crate::paths;
use crate::utils::{self, AttributeArgs};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{parse2, Error, FnArg, ItemFn, Path, Result, Type};

mod options;

use options::Options;

pub fn impl_command(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let mut fun = parse2::<ItemFn>(input)?;

    let names = if attr.is_empty() {
        vec![fun.sig.ident.to_string()]
    } else {
        parse2::<AttributeArgs>(attr)?.0
    };

    let (ctx_name, data, error) = utils::parse_generics(&fun.sig)?;
    let options = Options::parse(&mut fun.attrs)?;

    parse_arguments(ctx_name, &mut fun, &options)?;

    let builder_fn = builder_fn(&data, &error, &mut fun, names, &options);

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
    mut names: Vec<String>,
    options: &Options,
) -> TokenStream {
    let name = names.remove(0);
    let aliases = names;

    // Derive the name of the builder from the command function.
    // Prepend the command function's name with an underscore to avoid name
    // collisions.
    let builder_name = function.sig.ident.clone();
    let function_name = format_ident!("_{}", builder_name);
    function.sig.ident = function_name.clone();

    let command_builder = paths::command_builder_type();
    let command = paths::command_type(data, error);

    let vis = &function.vis;
    let external = &function.attrs;

    quote! {
        #(#external)*
        #vis fn #builder_name() -> #command {
            #command_builder::new(#name)
                #(.name(#aliases))*
                .function(#function_name)
                #options
                .build()
        }
    }
}

fn parse_arguments(ctx_name: Ident, function: &mut ItemFn, options: &Options) -> Result<()> {
    let mut arguments = Vec::new();

    let mut len = function.sig.inputs.len();
    while len > 2 {
        let argument = function.sig.inputs.pop().unwrap().into_value();

        arguments.push(Argument::new(argument)?);

        len -= 1;
    }

    if !arguments.is_empty() {
        arguments.reverse();

        check_arguments(&arguments)?;

        let delimiter = options.delimiter.as_ref().map_or(" ", String::as_str);
        let asegsty = paths::argument_segments_type();

        let b = &function.block;

        let argument_names = arguments.iter().map(|arg| &arg.name).collect::<Vec<_>>();
        let argument_tys = arguments.iter().map(|arg| &arg.ty).collect::<Vec<_>>();
        let argument_kinds = arguments.iter().map(|arg| &arg.kind).collect::<Vec<_>>();

        function.block = parse2(quote! {{
            let (#(#argument_names),*) = {
                // Place the segments into its scope to allow mutation of `Context::args`
                // afterwards, as `ArgumentSegments` holds a reference to the source string.
                let mut __args = #asegsty::new(&#ctx_name.args, #delimiter);

                #(let #argument_names: #argument_tys = #argument_kinds(&#ctx_name, &mut __args)?;)*

                (#(#argument_names),*)
            };

            #b
        }})?;
    }

    Ok(())
}

/// Returns a result indicating whether the list of arguments is valid.
///
/// Valid is defined as:
/// - a list of arguments that have required arguments first,
/// optional arguments second, and variadic arguments third; one or two of these
/// types of arguments can be missing.
/// - a list of arguments that only has one variadic argument parameter, if present.
fn check_arguments(args: &[Argument]) -> Result<()> {
    let mut last_arg: Option<&Argument> = None;

    for arg in args {
        if let Some(last_arg) = last_arg {
            match (last_arg.kind, arg.kind) {
                (ArgumentType::Optional, ArgumentType::Required) => {
                    return Err(Error::new(
                        last_arg.name.span(),
                        "optional argument cannot precede a required argument",
                    ));
                },
                (ArgumentType::Variadic, ArgumentType::Required) => {
                    return Err(Error::new(
                        last_arg.name.span(),
                        "variadic argument cannot precede a required argument",
                    ));
                },
                (ArgumentType::Variadic, ArgumentType::Optional) => {
                    return Err(Error::new(
                        last_arg.name.span(),
                        "variadic argument cannot precede an optional argument",
                    ));
                },
                (ArgumentType::Variadic, ArgumentType::Variadic) => {
                    return Err(Error::new(
                        arg.name.span(),
                        "a command cannot have two variadic argument parameters",
                    ));
                },
                (ArgumentType::Required, ArgumentType::Required)
                | (ArgumentType::Optional, ArgumentType::Optional)
                | (ArgumentType::Required, ArgumentType::Optional)
                | (ArgumentType::Required, ArgumentType::Variadic)
                | (ArgumentType::Optional, ArgumentType::Variadic) => {},
            };
        }

        last_arg = Some(arg);
    }

    Ok(())
}

struct Argument {
    name: Ident,
    ty: Box<Type>,
    kind: ArgumentType,
}

impl Argument {
    fn new(arg: FnArg) -> Result<Self> {
        let (name, ty) = utils::get_ident_and_type(&arg)?;
        let path = utils::get_path(&ty)?;

        let kind = ArgumentType::new(path);

        Ok(Self { name, ty, kind })
    }
}

#[derive(Clone, Copy)]
enum ArgumentType {
    Required,
    Optional,
    Variadic,
}

impl ArgumentType {
    fn new(path: &Path) -> Self {
        match path.segments.last().unwrap().ident.to_string().as_str() {
            "Option" => ArgumentType::Optional,
            "Vec" => ArgumentType::Variadic,
            _ => ArgumentType::Required,
        }
    }
}

impl ToTokens for ArgumentType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = match self {
            ArgumentType::Required => paths::required_argument_func(),
            ArgumentType::Optional => paths::optional_argument_func(),
            ArgumentType::Variadic => paths::variadic_arguments_func(),
        };

        tokens.extend(quote!(#path));
    }
}
