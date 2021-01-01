use crate::context::argument_segments_type;
use crate::context::{command_builder_type, command_fn, command_type, Context};
use crate::context::{opt_argument_func, req_argument_func, var_arguments_func};
use crate::utils::{self, AttributeArgs};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse2, FnArg, ItemFn, Path, Result, Stmt};

mod options;

use options::Options;

pub fn impl_command(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let mut fun = parse2::<ItemFn>(input)?;

    let names = if attr.is_empty() {
        vec![fun.sig.ident.to_string()]
    } else {
        parse2::<AttributeArgs>(attr)?.0
    };

    let ctx = Context::new(&fun)?;
    let options = Options::parse(&mut fun.attrs)?;

    parse_arguments(&ctx, &mut fun, &options)?;

    let builder_fn = builder_fn(&ctx, &mut fun, names, &options)?;
    let command_fn = command_fn(&ctx, &fun);

    let result = quote! {
        #builder_fn

        #command_fn
    };

    Ok(result)
}

fn builder_fn(
    ctx: &Context,
    function: &mut ItemFn,
    mut names: Vec<String>,
    options: &Options,
) -> Result<TokenStream> {
    let name = names.remove(0);
    let aliases = names;

    // Derive the name of the builder from the command function.
    // Prepend the command function's name with an underscore to avoid name
    // collisions.
    let builder_name = function.sig.ident.clone();
    let function_name = format_ident!("_{}", builder_name);
    function.sig.ident = function_name.clone();

    let command_builder = command_builder_type(ctx);
    let command = command_type(ctx);

    let vis = &function.vis;
    let external = &function.attrs;

    Ok(quote! {
        #(#external)*
        #vis fn #builder_name() -> #command {
            #command_builder::new(#name)
                #(.name(#aliases))*
                .function(#function_name)
                #options
                .build()
        }
    })
}

fn parse_arguments(ctx: &Context, function: &mut ItemFn, options: &Options) -> Result<()> {
    let mut len = function.sig.inputs.len();

    let mut arguments = Vec::new();

    let argument_segments = Ident::new("__args", Span::call_site());

    while len > 2 {
        let argument = function.sig.inputs.pop().unwrap().into_value();

        arguments.push(parse_argument(ctx, &argument_segments, argument)?);

        len -= 1;
    }

    if !arguments.is_empty() {
        let asegsty = argument_segments_type(ctx);
        let ctx_name = &ctx.ctx_name;
        let delimiter = options.delimiter.as_ref().map_or(" ", String::as_str);

        arguments.push(parse2(quote! {
            let mut #argument_segments = #asegsty::new(&#ctx_name.args, #delimiter);
        })?);

        arguments.reverse();

        let b = &function.block;

        function.block = parse2(quote! {{
            #(#arguments)*

            #b
        }})?;
    }

    Ok(())
}

fn parse_argument(ctx: &Context, asegs: &Ident, arg: FnArg) -> Result<Stmt> {
    let (name, t) = utils::get_ident_and_type(&arg)?;
    let p = utils::get_path(&t)?;

    let aty = get_argument_type(&p);

    let ctx_name = &ctx.ctx_name;
    let func = aty.func(ctx);

    let code = parse2(quote! {
        let #name: #t = #func(&#ctx_name, &mut #asegs)?;
    })?;

    Ok(code)
}

fn get_argument_type(p: &Path) -> ArgumentType {
    match p.segments.last().unwrap().ident.to_string().as_str() {
        "Option" => ArgumentType::Optional,
        "Vec" => ArgumentType::Variadic,
        _ => ArgumentType::Required,
    }
}

#[derive(Clone, Copy)]
enum ArgumentType {
    Required,
    Optional,
    Variadic,
}

impl ArgumentType {
    fn func(self, ctx: &Context) -> TokenStream {
        match self {
            ArgumentType::Required => req_argument_func(ctx),
            ArgumentType::Optional => opt_argument_func(ctx),
            ArgumentType::Variadic => var_arguments_func(ctx),
        }
    }
}
