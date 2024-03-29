//! Defines error types used by the framework.

use std::error::Error as StdError;
use std::fmt;

use crate::check::Reason;

/// An error describing why [`dispatch`]ing failed.
///
/// [`dispatch`]: crate::Framework::dispatch
#[derive(Debug, Clone)]
pub enum DispatchError {
    /// The message does not contain a command invocation.
    NormalMessage,
    /// The message only contains a prefix. Contains the prefix.
    PrefixOnly(String),
    /// The message contains a name not belonging to any command.
    InvalidCommandName(String),
    /// A check failed. Contains its name and the reasoning why it failed.
    CheckFailed(String, Reason),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DispatchError::NormalMessage => {
                write!(f, "message is normal")
            },
            DispatchError::PrefixOnly(prefix) => {
                write!(f, "only the prefix (`{}`) is present", prefix)
            },
            DispatchError::InvalidCommandName(name) => {
                write!(f, "name \"{}\" does not refer to any command", name)
            },
            DispatchError::CheckFailed(name, _) => write!(f, "\"{}\" check failed", name),
        }
    }
}

impl StdError for DispatchError {}

/// Returned when the call of [`dispatch`] fails.
///
/// [`dispatch`]: crate::Framework::dispatch
#[derive(Debug, Clone)]
pub enum Error<E> {
    /// Failed to dispatch a command.
    Dispatch(DispatchError),
    /// A command returned an error.
    User(E),
}

impl<E> From<DispatchError> for Error<E> {
    fn from(e: DispatchError) -> Self {
        Self::Dispatch(e)
    }
}

impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Dispatch(err) => fmt::Display::fmt(err, f),
            Error::User(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl<E: StdError + 'static> StdError for Error<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Dispatch(err) => Some(err),
            Error::User(err) => Some(err),
        }
    }
}
