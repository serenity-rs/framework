//! Utilities for parsing command arguments.

use crate::utils::ArgumentSegments;

use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

/// Error that might have occured when trying to parse an argument.
#[derive(Debug)]
pub enum ArgumentError<E> {
    /// Required argument is missing.
    ///
    /// This is only returned by the [`required_argument`] function.
    Missing,
    /// Parsing the argument failed.
    ///
    /// Contains the error from [`FromStr::Err`].
    Argument(E),
}

impl<E: fmt::Display> fmt::Display for ArgumentError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgumentError::Missing => f.write_str("missing required argument"),
            ArgumentError::Argument(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl<E: StdError + 'static> StdError for ArgumentError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ArgumentError::Argument(err) => Some(err),
            _ => None,
        }
    }
}

/// Takes a single segment from a list of segments and parses an argument out of it.
///
/// # Errors
///
/// - If the list of segments is empty, [`ArgumentError::Missing`] is returned.
/// - If the segment cannot be parsed into an argument, [`ArgumentError::Argument`] is
/// returned.
pub fn required_argument<T>(segments: &mut ArgumentSegments<'_>) -> Result<T, ArgumentError<T::Err>>
where
    T: FromStr,
{
    match segments.next() {
        Some(seg) => T::from_str(seg).map_err(ArgumentError::Argument),
        None => Err(ArgumentError::Missing),
    }
}

/// Tries to take a single segment from a list of segments and parse
/// an argument out of it.
///
/// If the list of segments is empty, `Ok(None)` is returned. Otherwise,
/// the first segment is taken and parsed into an argument. If parsing succeeds,
/// `Ok(Some(...))` is returned, otherwise `Err(...)`. The error is wrapped in
/// [`ArgumentError::Argument`].
pub fn optional_argument<T>(
    segments: &mut ArgumentSegments<'_>,
) -> Result<Option<T>, ArgumentError<T::Err>>
where
    T: FromStr,
{
    segments
        .next()
        .map(|seg| T::from_str(seg).map_err(ArgumentError::Argument))
        .transpose()
}

/// Tries to parse many arguments from a list of segments.
///
/// Each segment in the list is parsed into a vector of arguments. If parsing
/// all segments succeeds, the vector is returned. Otherwise, the first error
/// is returned. The error is wrapped in [`ArgumentError::Argument`].
pub fn variadic_arguments<T>(
    segments: &mut ArgumentSegments<'_>,
) -> Result<Vec<T>, ArgumentError<T::Err>>
where
    T: FromStr,
{
    segments
        .map(|seg| T::from_str(seg).map_err(ArgumentError::Argument))
        .collect()
}

/// Parses the remainder of the list of segments into an argument.
///
/// All segments (even if none) are concatenated to a single string
/// and parsed to the specified argument type. If parsing success,
/// `Ok(...)` is returned, otherwise `Err(...)`. The error is wrapped in
/// [`ArgumentError::Argument`].
pub fn rest_argument<T>(segments: &mut ArgumentSegments<'_>) -> Result<T, ArgumentError<T::Err>>
where
    T: FromStr,
{
    T::from_str(segments.source()).map_err(ArgumentError::Argument)
}
