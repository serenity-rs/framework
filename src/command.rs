use crate::{DefaultData, DefaultError};
use crate::utils::IdMap;
use crate::context::Context;

use serenity::model::channel::Message;
use serenity::futures::future::BoxFuture;

pub type CommandResult<E = DefaultError> = std::result::Result<(), E>;
pub type CommandFn<D = DefaultData, E = DefaultError> = fn(ctx: Context<D>, msg: Message) -> BoxFuture<'static, CommandResult<E>>;

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
    pub subcommands: IdMap<String, CommandId, Command<D, E>>,
}
