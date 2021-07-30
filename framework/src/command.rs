//! Functions and types relating to commands.
//!
//! A command is a function that performs work. It is invoked by a user on Discord.
//! It may have many names by which it can be invoked, but will always have at least
//! one name. It may possess subcommands to arrange functionality together. It may have
//! information that relays to the user what it does, what it is for, and how it
//! is used. It may have [`check`]s to allow/deny a user's access to the command.
//!
//! [`check`]: crate::check

use std::collections::HashSet;
use std::fmt;

use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;

use crate::check::{Check, CheckConstructor};
use crate::context::Context;
use crate::utils::IdMap;
use crate::DefaultError;

/// A function to dynamically create a string.
///
/// Used for [`Command::dynamic_description`] and [`Command::dynamic_usage`].
pub type StringHook<D, E> =
    for<'a> fn(ctx: &'a Context<D, E>, msg: &'a Message) -> BoxFuture<'a, Option<String>>;

/// A function to dynamically create a list of strings.
///
/// Used for [`Command::dynamic_examples`].
pub type StringsHook<D, E> =
    for<'a> fn(ctx: &'a Context<D, E>, msg: &'a Message) -> BoxFuture<'a, Vec<String>>;

/// [`IdMap`] for storing commands.
///
/// [`IdMap`]: crate::utils::IdMap
pub type CommandMap<D, E> = IdMap<String, CommandId, Command<D, E>>;

/// The result type of a [command function][fn].
///
/// [fn]: CommandFn
pub type CommandResult<T = (), E = DefaultError> = std::result::Result<T, E>;

/// The definition of a command function.
pub type CommandFn<D, E> =
    for<'a> fn(Context<D, E>, &'a Message) -> BoxFuture<'a, CommandResult<(), E>>;

/// A constructor of the [`Command`] type provided by the consumer of the framework.
pub type CommandConstructor<D, E> = fn() -> Command<D, E>;

/// A unique identifier of a [`Command`] stored in the [`CommandMap`].
///
/// It is constructed from [`CommandConstructor`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandId(pub(crate) usize);

impl CommandId {
    /// Converts the identifier to its internal representation.
    pub fn into_usize(self) -> usize {
        self.0
    }

    /// Converts the identifier to the constructor it points to.
    pub(crate) fn into_constructor<D, E>(self) -> CommandConstructor<D, E> {
        // SAFETY: CommandId in user code can only be constructed by its
        // `From<CommandConstructor<D, E>>` impl. This makes the transmute safe.

        unsafe { std::mem::transmute(self.0 as *const ()) }
    }
}

impl<D, E> From<CommandConstructor<D, E>> for CommandId {
    fn from(f: CommandConstructor<D, E>) -> Self {
        Self(f as usize)
    }
}

/// Data surrounding a command.
///
/// Refer to the [module-level documentation][docs].
///
/// [docs]: index.html
#[non_exhaustive]
pub struct Command<D, E> {
    /// The identifier of this command.
    pub id: CommandId,
    /// The function of this command.
    pub function: CommandFn<D, E>,
    /// The names of this command by which it can be invoked.
    pub names: Vec<String>,
    /// The subcommands belonging to this command.
    pub subcommands: HashSet<CommandId>,
    /// A string describing this command.
    pub description: Option<String>,
    /// A function to dynamically describe this command.
    pub dynamic_description: Option<StringHook<D, E>>,
    /// A string to express usage of this command.
    pub usage: Option<String>,
    /// A function to dynamically express usage of this command.
    pub dynamic_usage: Option<StringHook<D, E>>,
    /// A list of strings demonstrating usage of this command.
    pub examples: Vec<String>,
    /// A function to dynamically demonstrate usage of this command.
    pub dynamic_examples: Option<StringsHook<D, E>>,
    /// A boolean to indicate whether the command can be shown in help commands.
    pub help_available: bool,
    /// A function that allows/denies access to this command.
    pub check: Option<Check<D, E>>,
}

impl<D, E> Clone for Command<D, E> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            function: self.function,
            names: self.names.clone(),
            subcommands: self.subcommands.clone(),
            description: self.description.clone(),
            dynamic_description: self.dynamic_description,
            usage: self.usage.clone(),
            dynamic_usage: self.dynamic_usage,
            examples: self.examples.clone(),
            dynamic_examples: self.dynamic_examples,
            help_available: self.help_available,
            check: self.check.clone(),
        }
    }
}

impl<D, E> Default for Command<D, E> {
    fn default() -> Self {
        Self {
            id: CommandId::from((|| Command::default()) as CommandConstructor<D, E>),
            function: |_, _| Box::pin(async { Ok(()) }),
            names: Vec::default(),
            subcommands: HashSet::default(),
            description: None,
            dynamic_description: None,
            usage: None,
            dynamic_usage: None,
            examples: Vec::default(),
            dynamic_examples: None,
            help_available: true,
            check: None,
        }
    }
}

impl<D, E> fmt::Debug for Command<D, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("id", &self.id)
            .field("function", &"<fn>")
            .field("names", &self.names)
            .field("subcommands", &self.subcommands)
            .field("description", &self.description)
            .field("dynamic_description", &"<fn>")
            .field("usage", &self.usage)
            .field("dynamic_usage", &"<fn>")
            .field("examples", &self.examples)
            .field("dynamic_examples", &"<fn>")
            .field("help_available", &self.help_available)
            .field("check", &self.check)
            .finish()
    }
}

impl<D, E> Command<D, E> {
    /// Constructs a builder that will be used to create a command from scratch.
    ///
    /// Argument is the main name of the command.
    pub fn builder<I>(name: I) -> CommandBuilder<D, E>
    where
        I: Into<String>,
    {
        CommandBuilder::new(name)
    }
}

/// A builder type for creating a [`Command`] from scratch.
pub struct CommandBuilder<D, E> {
    inner: Command<D, E>,
}

impl<D, E> CommandBuilder<D, E> {
    /// Constructs a new instance of the builder.
    ///
    /// Argument is the main name of the command.
    pub fn new<I>(name: I) -> Self
    where
        I: Into<String>,
    {
        Self::default().name(name)
    }

    /// Assigns a name to this command.
    ///
    /// The name is added to the [`names`] list.
    ///
    /// [`names`]: Command::names
    pub fn name<I>(mut self, name: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.names.push(name.into());
        self
    }

    /// Assigns the function to this command.
    pub fn function(mut self, f: CommandFn<D, E>) -> Self {
        self.inner.function = f;
        self
    }

    /// Assigns a subcommand to this command.
    ///
    /// The subcommand is added to the [`subcommands`] list.
    ///
    /// [`subcommands`]: Command::subcommands
    pub fn subcommand(mut self, subcommand: CommandConstructor<D, E>) -> Self {
        self.inner.subcommands.insert(CommandId::from(subcommand));
        self
    }

    /// Assigns a static description to this command.
    pub fn description<I>(mut self, description: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.description = Some(description.into());

        self
    }

    /// Assigns a function to dynamically create a description to this command.
    pub fn dynamic_description(mut self, hook: StringHook<D, E>) -> Self {
        self.inner.dynamic_description = Some(hook);
        self
    }

    /// Assigns a static usage to this command.
    pub fn usage<I>(mut self, usage: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.usage = Some(usage.into());
        self
    }

    /// Assigns a function to dynamically create a usage to this command.
    pub fn dynamic_usage(mut self, hook: StringHook<D, E>) -> Self {
        self.inner.dynamic_usage = Some(hook);
        self
    }

    /// Assigns a static example of usage to this command.
    ///
    /// The example is added to the [`examples`] list.
    ///
    /// [`examples`]: Command::examples
    pub fn example<I>(mut self, example: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.examples.push(example.into());
        self
    }

    /// Assigns a function to dynamically create a list of examples to this command.
    pub fn dynamic_examples(mut self, hook: StringsHook<D, E>) -> Self {
        self.inner.dynamic_examples = Some(hook);
        self
    }

    /// Assigns a [`check`] function to this command.
    ///
    /// [`check`]: crate::check
    pub fn check(mut self, check: CheckConstructor<D, E>) -> Self {
        self.inner.check = Some(check());
        self
    }

    /// Complete building a command.
    ///
    /// # Panics
    ///
    /// This function may panic if:
    ///
    /// - The command that is about to be built is missing names.
    pub fn build(self) -> Command<D, E> {
        assert!(!self.inner.names.is_empty(), "a command must have at least one name");

        self.inner
    }
}

impl<D, E> Default for CommandBuilder<D, E> {
    fn default() -> Self {
        Self {
            inner: Command::default(),
        }
    }
}

impl<D, E> Clone for CommandBuilder<D, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<D, E> fmt::Debug for CommandBuilder<D, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandBuilder").field("inner", &self.inner).finish()
    }
}
