use crate::command::{CommandConstructor, CommandId, CommandMap};
use crate::context::PrefixContext;
use crate::group::{Group, GroupConstructor, GroupId, GroupMap};
use crate::{DefaultData, DefaultError};

use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;
use serenity::model::id::{ChannelId, GuildId, UserId};

use std::collections::HashSet;
use std::fmt;

pub type DynamicPrefix<D, E> =
    for<'a> fn(ctx: PrefixContext<'_, D, E>, msg: &'a Message) -> BoxFuture<'a, Option<usize>>;

#[derive(Debug, Default, Clone)]
pub struct BlockedEntities {
    pub channels: HashSet<ChannelId>,
    pub guilds: HashSet<GuildId>,
    pub users: HashSet<UserId>,
    pub commands: HashSet<CommandId>,
    pub groups: HashSet<GroupId>,
}

#[non_exhaustive]
pub struct Configuration<D = DefaultData, E = DefaultError> {
    pub prefixes: Vec<String>,
    pub dynamic_prefix: Option<DynamicPrefix<D, E>>,
    pub owners: Vec<UserId>,
    pub case_insensitive: bool,
    pub no_dm_prefix: bool,
    pub on_mention: Option<String>,
    pub blocked_entities: BlockedEntities,
    pub groups: GroupMap<D, E>,
    pub top_level_groups: Vec<Group<D, E>>,
    pub commands: CommandMap<D, E>,
}

impl<D, E> Clone for Configuration<D, E> {
    fn clone(&self) -> Self {
        Self {
            prefixes: self.prefixes.clone(),
            dynamic_prefix: self.dynamic_prefix,
            owners: self.owners.clone(),
            case_insensitive: self.case_insensitive,
            no_dm_prefix: self.no_dm_prefix,
            on_mention: self.on_mention.clone(),
            blocked_entities: self.blocked_entities.clone(),
            groups: self.groups.clone(),
            top_level_groups: self.top_level_groups.clone(),
            commands: self.commands.clone(),
        }
    }
}

impl<D, E> Default for Configuration<D, E> {
    fn default() -> Self {
        Self {
            prefixes: Vec::default(),
            dynamic_prefix: None,
            owners: Vec::default(),
            case_insensitive: false,
            no_dm_prefix: false,
            on_mention: None,
            blocked_entities: BlockedEntities::default(),
            groups: GroupMap::default(),
            top_level_groups: Vec::default(),
            commands: CommandMap::default(),
        }
    }
}

impl<D, E> Configuration<D, E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prefix<I>(&mut self, prefix: I) -> &mut Self
    where
        I: Into<String>,
    {
        self.prefixes.push(prefix.into());
        self
    }

    pub fn dynamic_prefix(&mut self, prefix: DynamicPrefix<D, E>) -> &mut Self {
        self.dynamic_prefix = Some(prefix);
        self
    }

    pub fn owner<I>(&mut self, owner: I) -> &mut Self
    where
        I: Into<UserId>,
    {
        self.owners.push(owner.into());
        self
    }

    pub fn case_insensitive(&mut self, b: bool) -> &mut Self {
        self.case_insensitive = b;

        self
    }

    pub fn no_dm_prefix(&mut self, b: bool) -> &mut Self {
        self.no_dm_prefix = b;
        self
    }

    pub fn on_mention(&mut self, id: Option<UserId>) -> &mut Self {
        self.on_mention = id.map(|id| id.to_string());
        self
    }

    pub fn blocked_entities(&mut self, blocked_entities: BlockedEntities) -> &mut Self {
        self.blocked_entities = blocked_entities;
        self
    }

    fn _group(&mut self, group: Group<D, E>) -> &mut Self {
        for prefix in &group.prefixes {
            let prefix = if self.case_insensitive {
                prefix.to_lowercase()
            } else {
                prefix.clone()
            };

            self.groups.insert_name(prefix, group.id);
        }

        for id in &group.subgroups {
            // SAFETY: GroupId in user code can only be constructed by its
            // `From<GroupConstructor>` impl. This makes the transmute safe.
            let constructor: GroupConstructor<D, E> =
                unsafe { std::mem::transmute(id.0 as *const ()) };

            let mut subgroup = constructor();
            subgroup.id = *id;
            self._group(subgroup);
        }

        for id in &group.commands {
            // SAFETY: CommandId in user code can only be constructed by its
            // `From<CommandConstructor<D, E>>` impl. This makes the transmute safe.
            let constructor: CommandConstructor<D, E> =
                unsafe { std::mem::transmute(id.0 as *const ()) };

            self.command(constructor);
        }

        self.groups.insert(group.id, group);

        self
    }

    pub fn group(&mut self, group: GroupConstructor<D, E>) -> &mut Self {
        let id = GroupId::from(group);

        let mut group = group();
        group.id = id;

        if group.prefixes.is_empty() {
            assert!(
                group.subgroups.is_empty(),
                "top level groups must not have prefixes nor subgroups"
            );

            self.top_level_groups.push(group);
            return self;
        }

        self._group(group)
    }

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

            self.commands.insert_name(name, id);
        }

        for id in &command.subcommands {
            // SAFETY: CommandId in user code can only be constructed by its
            // `From<CommandConstructor<D, E>>` impl. This makes the transmute safe.
            let constructor: CommandConstructor<D, E> =
                unsafe { std::mem::transmute(id.0 as *const ()) };

            self.command(constructor);
        }

        self.commands.insert(id, command);
        self
    }
}

impl<D, E> fmt::Debug for Configuration<D, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Configuration")
            .field("prefixes", &self.prefixes)
            .field("dynamic_prefix", &"<fn>")
            .field("owners", &self.owners)
            .field("case_insensitive", &self.case_insensitive)
            .field("no_dm_prefix", &self.no_dm_prefix)
            .field("on_mention", &self.on_mention)
            .field("blocked_entities", &self.blocked_entities)
            .field("groups", &self.groups)
            .field("top_level_groups", &self.top_level_groups)
            .field("commands", &self.commands)
            .finish()
    }
}
