use crate::utils::parse_bool;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, Result};

use std::convert::TryInto;

#[derive(Default)]
pub struct Options {
    check_in_help: Option<bool>,
    display_in_help: Option<bool>,
}

impl Options {
    pub fn parse(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut options = Self::default();

        let mut i = 0;

        while i < attrs.len() {
            let attr = &attrs[i];
            let name = attr.path.get_ident().unwrap().to_string();

            match name.as_str() {
                "check_in_help" => options.check_in_help = Some(parse_bool(&attr.try_into()?)?),
                "display_in_help" => options.display_in_help = Some(parse_bool(&attr.try_into()?)?),
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
            check_in_help,
            display_in_help,
        } = self;

        if let Some(check) = check_in_help {
            tokens.extend(quote!(.check_in_help(#check)));
        }

        if let Some(display) = display_in_help {
            tokens.extend(quote!(.display_in_help(#display)));
        }
    }
}
