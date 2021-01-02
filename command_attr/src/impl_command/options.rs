use crate::utils::{parse_bool, parse_identifier, parse_identifiers, parse_string};

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Result};

use std::convert::TryInto;

#[derive(Default)]
pub struct Options {
    subcommands: Vec<Ident>,
    description: Option<String>,
    dynamic_description: Option<Ident>,
    usage: Option<String>,
    dynamic_usage: Option<Ident>,
    examples: Vec<String>,
    dynamic_examples: Option<Ident>,
    help_available: Option<bool>,
    check: Option<Ident>,
    pub delimiter: Option<String>,
}

impl Options {
    pub fn parse(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut options = Self::default();

        let mut i = 0;

        while i < attrs.len() {
            let attr = &attrs[i];
            let name = attr.path.get_ident().unwrap().to_string();

            match name.as_str() {
                "doc" | "description" => {
                    let desc = options.description.get_or_insert_with(String::new);

                    if !desc.is_empty() {
                        desc.push('\n');
                    }

                    let mut s = parse_string(&attr.try_into()?)?;

                    if s.starts_with(' ') {
                        s.remove(0);
                    }

                    desc.push_str(&s);
                },
                "subcommands" => options.subcommands = parse_identifiers(&attr.try_into()?)?,
                "dynamic_description" =>
                    options.dynamic_description = Some(parse_identifier(&attr.try_into()?)?),
                "usage" => options.usage = Some(parse_string(&attr.try_into()?)?),
                "dynamic_usage" =>
                    options.dynamic_usage = Some(parse_identifier(&attr.try_into()?)?),
                "example" => options.examples.push(parse_string(&attr.try_into()?)?),
                "dynamic_examples" =>
                    options.dynamic_examples = Some(parse_identifier(&attr.try_into()?)?),
                "help_available" => options.help_available = Some(parse_bool(&attr.try_into()?)?),
                "check" => options.check = Some(parse_identifier(&attr.try_into()?)?),
                "delimiter" => options.delimiter = Some(parse_string(&attr.try_into()?)?),
                _ => {
                    i += 1;

                    continue;
                },
            }

            attrs.remove(i);
        }

        Ok(options)
    }
}

impl ToTokens for Options {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Options {
            subcommands,
            description,
            dynamic_description,
            usage,
            dynamic_usage,
            examples,
            dynamic_examples,
            help_available,
            check,
            ..
        } = self;

        tokens.extend(quote! {
            #(.subcommand(#subcommands))*
        });

        if let Some(desc) = description {
            tokens.extend(quote!(.description(#desc)));
        }

        if let Some(dyn_desc) = dynamic_description {
            tokens.extend(quote!(.dynamic_description(#dyn_desc)));
        }

        if let Some(usage) = usage {
            tokens.extend(quote!(.usage(#usage)));
        }

        if let Some(dyn_usage) = dynamic_usage {
            tokens.extend(quote!(.dynamic_usage(#dyn_usage)));
        }

        tokens.extend(quote! {
            #(.example(#examples))*
        });

        if let Some(dyn_examples) = dynamic_examples {
            tokens.extend(quote!(.dynamic_examples(#dyn_examples)));
        }

        if let Some(help_available) = help_available {
            tokens.extend(quote!(.help_available(#help_available)));
        }

        if let Some(check) = check {
            tokens.extend(quote!(.check(#check)));
        }
    }
}
