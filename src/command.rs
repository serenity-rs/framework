use crate::context::Context;
use crate::utils::IdMap;
use crate::{DefaultData, DefaultError};

use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;

use std::collections::HashSet;

pub type CommandMap<D = DefaultData, E = DefaultError> = IdMap<String, CommandId, Command<D, E>>;

pub type CommandResult<T = (), E = DefaultError> = std::result::Result<T, E>;
pub type CommandFn<D = DefaultData, E = DefaultError> =
    fn(ctx: Context<D, E>, msg: Message) -> BoxFuture<'static, CommandResult<(), E>>;

pub type CommandConstructor<D = DefaultData, E = DefaultError> = fn() -> Command<D, E>;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandId(pub(crate) usize);

impl CommandId {
    pub fn into_usize(self) -> usize {
        self.0
    }
}

impl<D, E> From<CommandConstructor<D, E>> for CommandId {
    fn from(f: CommandConstructor<D, E>) -> Self {
        Self(f as usize)
    }
}

#[derive(Debug, Clone)]
pub struct Command<D = DefaultData, E = DefaultError> {
    pub id: CommandId,
    pub function: CommandFn<D, E>,
    pub names: Vec<String>,
    pub subcommands: HashSet<CommandId>,
}

impl<D, E> Command<D, E> {
    pub fn builder<I>(name: I) -> CommandBuilder<D, E>
    where
        I: Into<String>,
    {
        CommandBuilder::new(name)
    }
}

impl<D, E> Default for Command<D, E> {
    fn default() -> Self {
        Self {
            id: CommandId::default(),
            function: |_, _| Box::pin(async { Ok(()) }),
            names: Vec::default(),
            subcommands: HashSet::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandBuilder<D = DefaultData, E = DefaultError> {
    inner: Command<D, E>,
}

impl<D, E> Default for CommandBuilder<D, E> {
    fn default() -> Self {
        Self {
            inner: Command::default(),
        }
    }
}

impl<D, E> CommandBuilder<D, E> {
    pub fn new<I>(name: I) -> Self
    where
        I: Into<String>,
    {
        Self::default().name(name)
    }

    pub fn name<I>(mut self, name: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.names.push(name.into());
        self
    }

    pub fn names<I>(mut self, names: impl IntoIterator<Item = I>) -> Self
    where
        I: Into<String>,
    {
        for name in names {
            self = self.name(name);
        }

        self
    }

    pub fn function(mut self, f: CommandFn<D, E>) -> Self {
        self.inner.function = f;
        self
    }

    pub fn subcommand(mut self, subcommand: CommandConstructor<D, E>) -> Self {
        self.inner.subcommands.insert(CommandId::from(subcommand));
        self
    }

    pub fn build(self) -> Command<D, E> {
        self.inner
    }
}
