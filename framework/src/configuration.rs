//! Configuration of the framework.

use crate::category::Category;
use crate::command::{CommandConstructor, CommandId, CommandMap};
use crate::context::PrefixContext;
use crate::{DefaultData, DefaultError};

use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;
use serenity::model::id::UserId;

use std::collections::HashSet;
use std::fmt;

/// The definition of the dynamic prefix hook.
pub type DynamicPrefix<D, E> =
    for<'a> fn(ctx: PrefixContext<'_, D, E>, msg: &'a Message) -> BoxFuture<'a, Option<usize>>;

/// The configuration of the framework.
#[non_exhaustive]
pub struct Configuration<D = DefaultData, E = DefaultError> {
    /// A list of static prefixes.
    pub prefixes: Vec<String>,
    /// A function to dynamically parse the prefix.
    pub dynamic_prefix: Option<DynamicPrefix<D, E>>,
    /// A boolean indicating whether casing of the letters in static prefixes,
    /// or command names does not matter.
    pub case_insensitive: bool,
    /// A boolean indicating whether the prefix is not necessary in direct messages.
    pub no_dm_prefix: bool,
    /// A user id of the bot that is used to compare mentions in prefix position.
    ///
    /// If filled, this allows for invoking commands by mentioning the bot.
    pub on_mention: Option<String>,
    /// A list of [`Category`]s.
    ///
    /// [`Category`]: crate::category::Category
    pub categories: Vec<Category>,
    /// A set of commands that can only appear at the beginning of a command invocation.
    pub root_level_commands: HashSet<CommandId>,
    /// An [`IdMap`] containing all [`Command`]s.
    ///
    /// [`IdMap`]: crate::utils::IdMap
    /// [`Command`]: crate::command::Command
    pub commands: CommandMap<D, E>,
}

impl<D, E> Clone for Configuration<D, E> {
    fn clone(&self) -> Self {
        Self {
            prefixes: self.prefixes.clone(),
            dynamic_prefix: self.dynamic_prefix,
            case_insensitive: self.case_insensitive,
            no_dm_prefix: self.no_dm_prefix,
            on_mention: self.on_mention.clone(),
            categories: self.categories.clone(),
            root_level_commands: self.root_level_commands.clone(),
            commands: self.commands.clone(),
        }
    }
}

impl<D, E> Default for Configuration<D, E> {
    fn default() -> Self {
        Self {
            prefixes: Vec::default(),
            dynamic_prefix: None,
            case_insensitive: false,
            no_dm_prefix: false,
            on_mention: None,
            categories: Vec::default(),
            root_level_commands: HashSet::default(),
            commands: CommandMap::default(),
        }
    }
}

impl<D, E> Configuration<D, E> {
    /// Creates a new instance of the framework configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns a prefix to this configuration.
    ///
    /// The prefix is added to the [`prefixes`] list.
    ///
    /// [`prefixes`]: Self::prefix
    pub fn prefix<I>(&mut self, prefix: I) -> &mut Self
    where
        I: Into<String>,
    {
        self.prefixes.push(prefix.into());
        self
    }

    /// Assigns a function to dynamically parse the prefix.
    pub fn dynamic_prefix(&mut self, prefix: DynamicPrefix<D, E>) -> &mut Self {
        self.dynamic_prefix = Some(prefix);
        self
    }

    /// Assigns a boolean indicating whether the casing of letters in static prefixes,
    /// or command names does not matter.
    pub fn case_insensitive(&mut self, b: bool) -> &mut Self {
        self.case_insensitive = b;

        self
    }

    /// Assigns a boolean indicating whether the prefix is not necessary in
    /// direct messages.
    pub fn no_dm_prefix(&mut self, b: bool) -> &mut Self {
        self.no_dm_prefix = b;
        self
    }

    /// Assigns a user id of the bot that will allow for mentions in prefix position.
    pub fn on_mention<I>(&mut self, id: I) -> &mut Self
    where
        I: Into<UserId>,
    {
        self.on_mention = Some(id.into().to_string());
        self
    }

    /// Assigns a category to this configuration.
    ///
    /// The category is added to the [`categories`] list. Additionally,
    /// all of its commands [are added][cmd] to the [`commands`] map
    ///
    /// [`categories`]: Self::categories
    /// [`commands`]: Self::commands
    /// [cmd]: Self::command
    pub fn category<I>(&mut self, name: I, cmds: &[CommandConstructor<D, E>]) -> &mut Self
    where
        I: Into<String>,
    {
        let mut commands = Vec::with_capacity(cmds.len());

        for cmd in cmds {
            self.command(*cmd);
            commands.push(CommandId::from(*cmd));
        }

        self.categories.push(Category {
            name: name.into(),
            commands,
        });

        self
    }

    /// Assigns a command to this configuration.
    ///
    /// The command is added to the [`commands`] map, alongside its subcommands.
    /// It it also added into the [`root_level_commands`] set.
    ///
    /// [`commands`]: Self::commands
    /// [`root_level_commands`]: Self::root_level_commands
    pub fn command(&mut self, command: CommandConstructor<D, E>) -> &mut Self {
        let id = CommandId::from(command);

        // Skip instantiating this root command if if already exists.
        if self.root_level_commands.contains(&id) {
            return self;
        }

        self.root_level_commands.insert(id);
        self._command(id, command);
        self
    }

    fn _subcommand(&mut self, command: CommandConstructor<D, E>) {
        let id = CommandId::from(command);

        // Skip instantiating this subcommand if it already exists.
        if self.commands.contains_id(id) {
            return;
        }

        self._command(id, command);
    }

    fn _command(&mut self, id: CommandId, command: CommandConstructor<D, E>) {
        let mut command = command();
        command.id = id;

        for name in &command.names {
            let name = if self.case_insensitive {
                name.to_lowercase()
            } else {
                name.clone()
            };

            self.commands.insert_name(name, command.id);
        }

        for id in &command.subcommands {
            let ctor: CommandConstructor<D, E> = id.into_constructor();
            self._subcommand(ctor);
        }

        self.commands.insert(command.id, command);
    }
}

impl<D, E> fmt::Debug for Configuration<D, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Configuration")
            .field("prefixes", &self.prefixes)
            .field("dynamic_prefix", &"<fn>")
            .field("case_insensitive", &self.case_insensitive)
            .field("no_dm_prefix", &self.no_dm_prefix)
            .field("on_mention", &self.on_mention)
            .field("categories", &self.categories)
            .field("root_level_commands", &self.root_level_commands)
            .field("commands", &self.commands)
            .finish()
    }
}
