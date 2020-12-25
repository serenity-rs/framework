use crate::context::Context;
use crate::{DefaultData, DefaultError};

use std::str::FromStr;

/// Abstraction for parsing arguments from a source.
pub trait Argument<D = DefaultData, E = DefaultError>: Sized {
    /// Type of error that may be returned from trying to parse a source.
    type Error;

    /// Parses a source into the argument type, with auxiliary information from
    /// a [`Context`].
    ///
    /// [`Context`]: crate::context::Context
    fn parse(ctx: &Context<D, E>, source: &str) -> Result<Self, Self::Error>;
}

impl<T, D, E> Argument<D, E> for T
where
    T: FromStr
{
    type Error = <T as FromStr>::Err;

    fn parse(_: &Context<D, E>, source: &str) -> Result<Self, Self::Error> {
        <T as FromStr>::from_str(source)
    }
}
