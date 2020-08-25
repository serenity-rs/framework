use crate::command::CommandId;
use crate::group::GroupId;
use crate::DefaultError;

use serenity::model::id::{ChannelId, GuildId, UserId};

use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum DispatchError {
    /// A channel is blocked. Contains the ID of the channel.
    BlockedChannel(ChannelId),
    /// A guild is blocked. Contains the ID of the guild.
    BlockedGuild(GuildId),
    /// A user is blocked. Contains the ID of the user.
    BlockedUser(UserId),
    /// A command is blocked. Contains the ID of the command.
    BlockedCommand(CommandId),
    /// A group is blocked. Contains the ID of the group.
    BlockedGroup(GroupId),
    /// The message does not contain a command invocation.
    NormalMessage,
    /// The message only contains a prefix.
    PrefixOnly,
    /// The message is missing information needed to complete its command invocation.
    MissingContent,
    /// An invalid name for a command was passed.
    InvalidCommandName(String),
    /// An invalid command was passed. Contains the ID to the group, which the
    /// command, whose ID is also contained, does not belong to.
    InvalidCommand(GroupId, CommandId),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DispatchError::BlockedChannel(id) => {
                write!(f, "failed to dispatch because channel {} is blocked", id.0)
            }
            DispatchError::BlockedGuild(id) => {
                write!(f, "failed to dispatch because guild {} is blocked", id.0)
            }
            DispatchError::BlockedUser(id) => {
                write!(f, "failed to dispatch because user {} is blocked", id.0)
            }
            DispatchError::BlockedCommand(id) => write!(
                f,
                "failed to dispatch because command {} is blocked",
                id.into_usize()
            ),
            DispatchError::BlockedGroup(id) => write!(
                f,
                "failed to dispatch because group {} is blocked",
                id.into_usize()
            ),
            DispatchError::NormalMessage => {
                write!(f, "failed to dispatch because the message is normal")
            }
            DispatchError::PrefixOnly => {
                write!(f, "failed to dispatch because only the prefix is present")
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
            DispatchError::InvalidCommand(group, command) => write!(
                f,
                "failed to dispatch because command {} does not belong to group {}",
                group.into_usize(),
                command.into_usize()
            ),
        }
    }
}

impl StdError for DispatchError {}

#[derive(Debug, Clone)]
pub enum Error<E = DefaultError> {
    Dispatch(DispatchError),
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
