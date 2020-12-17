//! Configuration of the framework.

use crate::command::{CommandConstructor, CommandId, CommandMap};
use crate::context::PrefixContext;
use crate::group::{GroupConstructor, GroupId, GroupMap};
use crate::{DefaultData, DefaultError};

use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;
use serenity::model::id::UserId;

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
    /// group prefixes, or command names does not matter.
    pub case_insensitive: bool,
    /// A boolean indicating whether the prefix is not necessary in direct messages.
    pub no_dm_prefix: bool,
    /// A user id of the bot that is used to compare mentions in prefix position.
    ///
    /// If filled, this allows for invoking commands by mentioning the bot.
    pub on_mention: Option<String>,
    /// An [`IdMap`] containing all [`Group`]s.
    ///
    /// [`IdMap`]: crate::utils::IdMap
    /// [`Group`]: crate::group::Group
    pub groups: GroupMap<D, E>,
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
            groups: self.groups.clone(),
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
            groups: GroupMap::default(),
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
    /// group prefixes or command names does not matter.
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

    /// Assigns a group to this configuration.
    ///
    /// The group is added to the [`groups`] map.
    ///
    /// [`groups`]: Self::groups
    pub fn group(&mut self, group: GroupConstructor<D, E>) -> &mut Self {
        let id = GroupId::from(group);

        let mut group = group();
        group.id = id;

        assert!(!group.prefixes.is_empty(), "groups cannot have no prefixes");

        for prefix in &group.prefixes {
            let prefix = if self.case_insensitive {
                prefix.to_lowercase()
            } else {
                prefix.clone()
            };

            self.groups.insert_name(prefix, group.id);
        }

        for id in &group.subgroups {
            let ctor: GroupConstructor<D, E> = id.into_constructor();
            self.group(ctor);
        }

        for id in &group.commands {
            let ctor: CommandConstructor<D, E> = id.into_constructor();
            self.command(ctor);
        }

        self.groups.insert(group.id, group);

        self
    }

    /// Assigns a command to this configuration.
    ///
    /// The command is added to the [`commands`] map.
    ///
    /// [`commands`]: Self::commands
    pub fn command(&mut self, command: CommandConstructor<D, E>) -> &mut Self {
        let id = CommandId::from(command);

        let mut command = command();
        command.id = id;

        assert!(!command.names.is_empty(), "command cannot have no names");

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
            self.command(ctor);
        }

        self.commands.insert(command.id, command);

        self
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
            .field("groups", &self.groups)
            .field("commands", &self.commands)
            .finish()
    }
}
