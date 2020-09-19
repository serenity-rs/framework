//! Defines error types used by the framework.

use crate::check::Reason;
use crate::command::CommandId;
use crate::group::GroupId;
use crate::DefaultError;

use std::error::Error as StdError;
use std::fmt;

/// An error describing why [`dispatch`]ing failed.
///
/// [`dispatch`]: ../struct.Framework.html#method.dispatch
#[derive(Debug, Clone)]
pub enum DispatchError {
    /// The message does not contain a command invocation.
    NormalMessage,
    /// The message only contains a prefix. Contains the prefix.
    PrefixOnly(String),
    /// The message is missing information needed to complete its command invocation.
    MissingContent,
    /// An invalid name for a command was passed.
    InvalidCommandName(String),
    /// An invalid command was passed. Contains the ID to the group (if it was present),
    /// which the command, whose ID is also contained, does not belong to.
    InvalidCommand(Option<GroupId>, CommandId),
    /// A check failed. Contains its name and the reasoning why it failed.
    CheckFailed(String, Reason),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DispatchError::NormalMessage => {
                write!(f, "failed to dispatch because the message is normal")
            }
            DispatchError::PrefixOnly(prefix) => {
                write!(f, "failed to dispatch because only the prefix (`{}`) is present", prefix)
            }
            DispatchError::MissingContent => write!(
                f,
                "failed to dispatch because the message content is missing information"
            ),
            DispatchError::InvalidCommandName(name) => write!(
                f,
                "failed to dispatch because \"{}\" is not a valid command",
                name
            ),
            DispatchError::InvalidCommand(Some(group), command) => write!(
                f,
                "failed to dispatch because command {} does not belong to group {}",
                group.into_usize(),
                command.into_usize()
            ),
            DispatchError::InvalidCommand(None, command) => write!(
                f,
                "failed to dispatch because command {} does not belong to any top-level group",
                command.into_usize()
            ),
            DispatchError::CheckFailed(name, _) => write!(
                f,
                "failed to dispatch because the \"{}\" check failed",
                name
            ),
        }
    }
}

impl StdError for DispatchError {}

/// Returned when the call of [`dispatch`] fails.
///
/// [`dispatch`]: ../struct.Framework.html#method.dispatch
#[derive(Debug, Clone)]
pub enum Error<E = DefaultError> {
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
