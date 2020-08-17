use crate::context::Context;
use crate::utils::IdMap;
use crate::{DefaultData, DefaultError};

use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;

pub type CommandMap<D = DefaultData, E = DefaultError> = IdMap<String, CommandId, Command<D, E>>;

pub type CommandResult<E = DefaultError> = std::result::Result<(), E>;
pub type CommandFn<D = DefaultData, E = DefaultError> =
    fn(ctx: Context<D>, msg: Message) -> BoxFuture<'static, CommandResult<E>>;

pub type CommandConstructor<D = DefaultData, E = DefaultError> = fn() -> Command<D, E>;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandId(pub u64);

impl<D, E> From<CommandConstructor<D, E>> for CommandId {
    fn from(f: CommandConstructor<D, E>) -> Self {
        Self(f as u64)
    }
}

#[derive(Debug, Clone)]
pub struct Command<D = DefaultData, E = DefaultError> {
    pub function: CommandFn<D, E>,
    pub names: Vec<String>,
    pub subcommands: CommandMap<D, E>,
}
