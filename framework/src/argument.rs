//! Utilities for parsing command arguments.

use std::error::Error as StdError;
use std::fmt;

use serenity::{async_trait, futures::TryFutureExt, model::prelude::*, prelude::*, utils::Parse};

use crate::utils::ArgumentSegments;

/// Error that might have occured when trying to parse an argument.
#[derive(Debug)]
pub enum ArgumentError<E> {
    /// Required argument is missing.
    ///
    /// This is only returned by the [`required_argument_from_str`] and [`required_argument_parse`]
    /// functions.
    Missing,
    /// Parsing the argument failed.
    ///
    /// Contains the error from [`serenity::utils::Parse::Err`].
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

/// Takes a single segment from a list of segments and parses an argument out of it using the
/// [std::str::FromStr] trait.
///
/// # Errors
///
/// - If the list of segments is empty, [`ArgumentError::Missing`] is returned.
/// - If the segment cannot be parsed into an argument, [`ArgumentError::Argument`] is
/// returned.
pub async fn required_argument_from_str<T>(
    _ctx: &Context,
    _msg: &Message,
    segments: &mut ArgumentSegments<'_>,
) -> Result<T, ArgumentError<T::Err>>
where
    T: std::str::FromStr,
{
    match segments.next() {
        Some(seg) => T::from_str(seg).map_err(ArgumentError::Argument),
        None => Err(ArgumentError::Missing),
    }
}

/// Takes a single segment from a list of segments and parses an argument out of it using the
/// [serenity::utils::Parse] trait.
///
/// # Errors
///
/// - If the list of segments is empty, [`ArgumentError::Missing`] is returned.
/// - If the segment cannot be parsed into an argument, [`ArgumentError::Argument`] is
/// returned.
pub async fn required_argument_parse<T>(
    ctx: &Context,
    msg: &Message,
    segments: &mut ArgumentSegments<'_>,
) -> Result<T, ArgumentError<T::Err>>
where
    T: Parse,
{
    match segments.next() {
        Some(seg) => T::parse(ctx, msg, seg).await.map_err(ArgumentError::Argument),
        None => Err(ArgumentError::Missing),
    }
}

/// Tries to take a single segment from a list of segments and parse
/// an argument out of it using the [std::str::FromStr] trait.
///
/// If the list of segments is empty, `Ok(None)` is returned. Otherwise,
/// the first segment is taken and parsed into an argument. If parsing succeeds,
/// `Ok(Some(...))` is returned, otherwise `Err(...)`. The error is wrapped in
/// [`ArgumentError::Argument`].
pub async fn optional_argument_from_str<T>(
    _ctx: &Context,
    _msg: &Message,
    segments: &mut ArgumentSegments<'_>,
) -> Result<Option<T>, ArgumentError<T::Err>>
where
    T: std::str::FromStr,
{
    match segments.next() {
        Some(seg) => T::from_str(seg).map(Some).map_err(ArgumentError::Argument),
        None => Ok(None),
    }
}

/// Tries to take a single segment from a list of segments and parse
/// an argument out of it using the [serenity::utils::Parse] trait.
///
/// If the list of segments is empty, `Ok(None)` is returned. Otherwise,
/// the first segment is taken and parsed into an argument. If parsing succeeds,
/// `Ok(Some(...))` is returned, otherwise `Err(...)`. The error is wrapped in
/// [`ArgumentError::Argument`].
pub async fn optional_argument_parse<T>(
    ctx: &Context,
    msg: &Message,
    segments: &mut ArgumentSegments<'_>,
) -> Result<Option<T>, ArgumentError<T::Err>>
where
    T: Parse,
{
    match segments.next() {
        Some(seg) => T::parse(ctx, msg, seg).await.map(Some).map_err(ArgumentError::Argument),
        None => Ok(None),
    }
}

/// Tries to parse many arguments from a list of segments using the [std::str::FromStr] trait.
///
/// Each segment in the list is parsed into a vector of arguments. If parsing
/// all segments succeeds, the vector is returned. Otherwise, the first error
/// is returned. The error is wrapped in [`ArgumentError::Argument`].
pub async fn variadic_arguments_from_str<T>(
    _ctx: &Context,
    _msg: &Message,
    segments: &mut ArgumentSegments<'_>,
) -> Result<Vec<T>, ArgumentError<T::Err>>
where
    T: std::str::FromStr,
{
    segments.map(|seg| T::from_str(seg).map_err(ArgumentError::Argument)).collect()
}

/// Tries to parse many arguments from a list of segments using the [serenity::utils::Parse] trait.
///
/// Each segment in the list is parsed into a vector of arguments. If parsing
/// all segments succeeds, the vector is returned. Otherwise, the first error
/// is returned. The error is wrapped in [`ArgumentError::Argument`].
pub async fn variadic_arguments_parse<T>(
    ctx: &Context,
    msg: &Message,
    segments: &mut ArgumentSegments<'_>,
) -> Result<Vec<T>, ArgumentError<T::Err>>
where
    T: Parse,
{
    serenity::futures::future::try_join_all(segments.map(|seg| T::parse(ctx, msg, seg)))
        .await
        .map_err(ArgumentError::Argument)
}

/// Parses the remainder of the list of segments into an argument using the [std::str::FromStr]
/// trait.
///
/// All segments (even if none) are concatenated to a single string
/// and parsed to the specified argument type. If parsing success,
/// `Ok(...)` is returned, otherwise `Err(...)`. The error is wrapped in
/// [`ArgumentError::Argument`].
pub async fn rest_argument_from_str<T>(
    _ctx: &Context,
    _msg: &Message,
    segments: &mut ArgumentSegments<'_>,
) -> Result<T, ArgumentError<T::Err>>
where
    T: std::str::FromStr,
{
    T::from_str(segments.source()).map_err(ArgumentError::Argument)
}

/// Parses the remainder of the list of segments into an argument using the [serenity::utils::Parse]
/// trait.
///
/// All segments (even if none) are concatenated to a single string
/// and parsed to the specified argument type. If parsing success,
/// `Ok(...)` is returned, otherwise `Err(...)`. The error is wrapped in
/// [`ArgumentError::Argument`].
pub async fn rest_argument_parse<T>(
    ctx: &Context,
    msg: &Message,
    segments: &mut ArgumentSegments<'_>,
) -> Result<T, ArgumentError<T::Err>>
where
    T: Parse,
{
    T::parse(ctx, msg, segments.source()).await.map_err(ArgumentError::Argument)
}

/// Denotes a type that can be either one of two different types.
///
/// It derives the [`Parse`] trait and can be used to parse an argument as either of two types.
/// It attempts to parse into the type that is indicated first. If parsing into the first type fails,
/// an attempt to parse into the second type is made. If both attempts fail, the overall parsing
/// fails and returns a [`ParseEitherError`].
///
/// This can also be used to handle larger combinations of types by chaining [`ParseEither`]s,
/// for example, `ParseEither<f32, ParseEither<i32, String>>`.
#[derive(Debug)]
#[non_exhaustive]
pub enum ParseEither<T, U>
where
    T: Parse,
    U: Parse,
{
    /// The first variant.
    VariantOne(T),
    /// The second variant.
    VariantTwo(U),
}

/// Error that is returned when [`ParseEither::parse`] fails.
#[non_exhaustive]
pub struct ParseEitherError<T, U>
where
    T: Parse,
    U: Parse,
{
    /// The error returned from parsing the first variant.
    pub err_one: T::Err,
    /// The error returned from parsing the second variant.
    pub err_two: U::Err,
}

impl<T, U> fmt::Debug for ParseEitherError<T, U>
where
    T: Parse,
    T::Err: fmt::Debug,
    U: Parse,
    U::Err: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseEitherError")
            .field("err_one", &self.err_one)
            .field("err_two", &self.err_two)
            .finish()
    }
}

impl<T, U> std::error::Error for ParseEitherError<T, U>
where
    T: Parse,
    T::Err: fmt::Debug + fmt::Display,
    U: Parse,
    U::Err: fmt::Debug + fmt::Display,
{
}

impl<T, U> fmt::Display for ParseEitherError<T, U>
where
    T: Parse,
    T::Err: fmt::Debug + fmt::Display,
    U: Parse,
    U::Err: fmt::Debug + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parsing into type one failed: {}\nParsing into type two failed: {}",
            self.err_one, self.err_two
        )
    }
}

#[async_trait]
impl<T, U> Parse for ParseEither<T, U>
where
    T: Parse,
    T::Err: Send,
    U: Parse,
{
    type Err = ParseEitherError<T, U>;

    async fn parse(ctx: &Context, msg: &Message, s: &str) -> Result<Self, Self::Err> {
        let parse_one = async { T::parse(ctx, msg, s).await.map(|v| Self::VariantOne(v)) };
        let parse_two = async { U::parse(ctx, msg, s).await.map(|v| Self::VariantTwo(v)) };

        parse_one
            .or_else(|e1| async {
                parse_two.await.map_err(|e2| Self::Err {
                    err_one: e1,
                    err_two: e2,
                })
            })
            .await
    }
}
