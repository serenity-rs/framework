//! Utilities for parsing command arguments.

use crate::context::Context;
use crate::utils::ArgumentSegments;
use crate::{DefaultData, DefaultError};

use std::error::Error as StdError;
use std::fmt;
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

/// Error that might have occured when trying to parse [a required argument][rarg].
///
/// [rarg]: req_argument
#[derive(Debug)]
pub enum RequiredArgumentError<E> {
    /// Required argument is missing.
    Missing,
    /// Parsing the argument failed.
    ///
    /// Contains the error from [`Argument::Error`].
    Argument(E),
}

impl<E: fmt::Display> fmt::Display for RequiredArgumentError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequiredArgumentError::Missing => f.write_str("missing required argument"),
            RequiredArgumentError::Argument(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl<E: StdError + 'static> StdError for RequiredArgumentError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            RequiredArgumentError::Argument(err) => Some(err),
            _ => None,
        }
    }
}

/// Takes a single segment from a list of segments and parses [an argument][arg] out of it.
///
/// # Errors
///
/// - If the list of segments is empty, [`RequiredArgumentError::Missing`] is returned.
/// - If the segment cannot be parsed into an argument, [`RequiredArgumentError::Argument`] is
/// returned.
///
/// [arg]: Argument
pub fn req_argument<T, D, E>(
    ctx: &Context<D, E>,
    segments: &mut ArgumentSegments<'_>,
) -> Result<T, RequiredArgumentError<T::Error>>
where
    T: Argument<D, E>,
{
    match segments.next() {
        Some(seg) => T::parse(ctx, seg).map_err(RequiredArgumentError::Argument),
        None => Err(RequiredArgumentError::Missing),
    }
}

/// Tries to take a single segment from a list of segments and
/// parse [an argument][arg] out of it.
///
/// If the list of segments is empty, `Ok(None)` is returned. Otherwise,
/// the first segment is taken and parsed into an argument. If parsing succeeds,
/// `Ok(Some(...))` is returned, otherwise `Err`.
///
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

/// Tries to parse many [arguments][arg] from a list of segments.
///
/// Each segment in the list is parsed into a vector of arguments. If parsing
/// all segments succeeds, the vector is returned. Otherwise, the first error
/// is returned.
///
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
