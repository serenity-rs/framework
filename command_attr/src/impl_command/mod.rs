use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse2, Attribute, Error, FnArg, ItemFn, Path, Result, Type};

use crate::paths;
use crate::utils::{self, AttributeArgs};

mod options;

use options::Options;

pub fn impl_command(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let mut fun = parse2::<ItemFn>(input)?;

    let names = if attr.is_empty() {
        vec![fun.sig.ident.to_string()]
    } else {
        parse2::<AttributeArgs>(attr)?.0
    };

    let (ctx_name, msg_name, data, error) = utils::parse_generics(&fun.sig)?;
    let options = Options::parse(&mut fun.attrs)?;

    parse_arguments(ctx_name, msg_name, &mut fun, &options)?;

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

fn parse_arguments(
    ctx_name: Ident,
    msg_name: Ident,
    function: &mut ItemFn,
    options: &Options,
) -> Result<()> {
    let mut arguments = Vec::new();

    while function.sig.inputs.len() > 2 {
        let argument = function.sig.inputs.pop().unwrap().into_value();

        arguments.push(Argument::new(argument)?);
    }

    if !arguments.is_empty() {
        arguments.reverse();

        check_arguments(&arguments)?;

        let delimiter = options.delimiter.as_ref().map_or(" ", String::as_str);
        let asegsty = paths::argument_segments_type();

        let b = &function.block;

        let argument_names = arguments.iter().map(|arg| &arg.name).collect::<Vec<_>>();
        let argument_tys = arguments.iter().map(|arg| &arg.ty).collect::<Vec<_>>();
        let argument_parsers = arguments.iter().map(|arg| &arg.parser).collect::<Vec<_>>();

        function.block = parse2(quote! {{
            let (#(#argument_names),*) = {
                // Place the segments into its scope to allow mutation of `Context::args`
                // afterwards, as `ArgumentSegments` holds a reference to the source string.
                let mut __args = #asegsty::new(&#ctx_name.args, #delimiter);

                #(let #argument_names: #argument_tys = #argument_parsers(
                    &#ctx_name.serenity_ctx,
                    &#msg_name,
                    &mut __args
                ).await?;)*

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
/// - a list of arguments that only has one rest argument parameter, if present.
/// - a list of arguments that only has one variadic argument parameter or one rest
/// argument parameter.
fn check_arguments(args: &[Argument]) -> Result<()> {
    let mut last_arg: Option<&Argument> = None;

    for arg in args {
        if let Some(last_arg) = last_arg {
            match (last_arg.parser.type_, arg.parser.type_) {
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
                (ArgumentType::Rest, ArgumentType::Required) => {
                    return Err(Error::new(
                        last_arg.name.span(),
                        "rest argument cannot precede a required argument",
                    ));
                },
                (ArgumentType::Rest, ArgumentType::Optional) => {
                    return Err(Error::new(
                        last_arg.name.span(),
                        "rest argument cannot precede an optional argument",
                    ));
                },
                (ArgumentType::Rest, ArgumentType::Variadic) => {
                    return Err(Error::new(
                        last_arg.name.span(),
                        "a rest argument cannot be used alongside a variadic argument",
                    ));
                },
                (ArgumentType::Variadic, ArgumentType::Rest) => {
                    return Err(Error::new(
                        last_arg.name.span(),
                        "a variadic argument cannot be used alongside a rest argument",
                    ));
                },
                (ArgumentType::Variadic, ArgumentType::Variadic) => {
                    return Err(Error::new(
                        arg.name.span(),
                        "a command cannot have two variadic argument parameters",
                    ));
                },
                (ArgumentType::Rest, ArgumentType::Rest) => {
                    return Err(Error::new(
                        arg.name.span(),
                        "a command cannot have two rest argument parameters",
                    ));
                },
                (ArgumentType::Required, ArgumentType::Required)
                | (ArgumentType::Optional, ArgumentType::Optional)
                | (ArgumentType::Required, ArgumentType::Optional)
                | (ArgumentType::Required, ArgumentType::Variadic)
                | (ArgumentType::Optional, ArgumentType::Variadic)
                | (ArgumentType::Required, ArgumentType::Rest)
                | (ArgumentType::Optional, ArgumentType::Rest) => {},
            };
        }

        last_arg = Some(arg);
    }

    Ok(())
}

struct Argument {
    name: Ident,
    ty: Box<Type>,
    parser: ArgumentParser,
}

impl Argument {
    fn new(arg: FnArg) -> Result<Self> {
        let binding = utils::get_pat_type(&arg)?;

        let name = utils::get_ident(&binding.pat)?;

        let ty = binding.ty.clone();

        let path = utils::get_path(&ty)?;
        let parser = ArgumentParser::new(&binding.attrs, path)?;

        Ok(Self {
            name,
            ty,
            parser,
        })
    }
}

#[derive(Clone, Copy)]
enum ArgumentType {
    Required,
    Optional,
    Variadic,
    Rest,
}

#[derive(Clone, Copy)]
struct ArgumentParser {
    type_: ArgumentType,
    use_parse_trait: bool,
}

impl ArgumentParser {
    fn new(attrs: &[Attribute], path: &Path) -> Result<Self> {
        let mut is_rest_argument = false;
        let mut use_parse_trait = false;
        for attr in attrs {
            let attr = utils::parse_attribute(attr)?;

            if attr.path.is_ident("rest") {
                is_rest_argument = true;

                if !attr.values.is_empty() {
                    return Err(Error::new(
                        attrs[0].span(),
                        "the `rest` attribute does not accept any input",
                    ));
                }
            } else if attr.path.is_ident("parse") {
                use_parse_trait = true;

                if !attr.values.is_empty() {
                    return Err(Error::new(
                        attrs[0].span(),
                        "the `parse` attribute does not accept any input",
                    ));
                }
            } else {
                return Err(Error::new(
                    attrs[0].span(),
                    "invalid attribute name, expected `rest` or `parse`",
                ));
            }
        }

        let type_ = if is_rest_argument {
            ArgumentType::Rest
        } else {
            match path.segments.last().unwrap().ident.to_string().as_str() {
                "Option" => ArgumentType::Optional,
                "Vec" => ArgumentType::Variadic,
                _ => ArgumentType::Required,
            }
        };

        Ok(Self {
            type_,
            use_parse_trait,
        })
    }
}

impl ToTokens for ArgumentParser {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = match (self.type_, self.use_parse_trait) {
            (ArgumentType::Required, false) => paths::required_argument_from_str_func(),
            (ArgumentType::Required, true) => paths::required_argument_parse_func(),
            (ArgumentType::Optional, false) => paths::optional_argument_from_str_func(),
            (ArgumentType::Optional, true) => paths::optional_argument_parse_func(),
            (ArgumentType::Variadic, false) => paths::variadic_arguments_from_str_func(),
            (ArgumentType::Variadic, true) => paths::variadic_arguments_parse_func(),
            (ArgumentType::Rest, false) => paths::rest_argument_from_str_func(),
            (ArgumentType::Rest, true) => paths::rest_argument_parse_func(),
        };

        tokens.extend(quote!(#path));
    }
}
