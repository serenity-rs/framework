//! Utilities for parsing command arguments.

use crate::context::Context;
use crate::utils::ArgumentSegments;
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
    T: FromStr,
{
    type Error = <T as FromStr>::Err;

    fn parse(_: &Context<D, E>, source: &str) -> Result<Self, Self::Error> {
        <T as FromStr>::from_str(source)
    }
}

/// Tries to take a single segment from [a list of segments][asegs] and parse
/// [an argument][arg] out of it.
///
/// If the list of segments is empty, `Ok(None)` is returned. Otherwise,
/// the first segment is taken and parsed into an argument. If parsing succeeds,
/// `Ok(Some(...))` is returned, otherwise `Err`.
///
/// [asegs]: crate::utils::ArgumentSegments
/// [arg]: Argument
pub fn opt_argument<T, D, E>(
    ctx: &Context<D, E>,
    segments: &mut ArgumentSegments<'_>,
) -> Result<Option<T>, T::Error>
where
    T: Argument<D, E>,
{
    segments.next().map(|seg| T::parse(ctx, seg)).transpose()
}

/// Tries to parse many [arguments][arg] from [a list of segments][asegs].
///
/// Each segment in the list is parsed into a vector of arguments. If parsing
/// all segments succeeds, the vector is returned. Otherwise, the first error
/// is returned.
///
/// [asegs]: crate::utils::ArgumentSegments
/// [arg]: Argument
pub fn var_arguments<T, D, E>(
    ctx: &Context<D, E>,
    segments: &mut ArgumentSegments<'_>,
) -> Result<Vec<T>, T::Error>
where
    T: Argument<D, E>,
{
    segments.map(|seg| T::parse(ctx, seg)).collect()
}
