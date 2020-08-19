use crate::{DefaultData, DefaultError};
use crate::command::CommandId;
use crate::group::{GroupId, Group, GroupMap, GroupConstructor};

use serenity::model::id::{ChannelId, GuildId, UserId};

#[derive(Debug, Default, Clone)]
pub struct BlockedEntities {
    pub channels: Vec<ChannelId>,
    pub guilds: Vec<GuildId>,
    pub users: Vec<UserId>,
    pub commands: Vec<CommandId>,
    pub groups: Vec<GroupId>,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Configuration<D = DefaultData, E = DefaultError> {
    pub prefixes: Vec<String>,
    pub owners: Vec<UserId>,
    pub case_insensitive: bool,
    pub no_dm_prefix: bool,
    pub on_mention: Option<String>,
    pub blocked_entities: BlockedEntities,
    pub groups: GroupMap<D, E>,
    pub top_level_groups: Vec<Group<D, E>>,
}

impl<D, E> Default for Configuration<D, E> {
    fn default() -> Self {
        Self {
            prefixes: Vec::default(),
            owners: Vec::default(),
            case_insensitive: false,
            no_dm_prefix: false,
            on_mention: None,
            blocked_entities: BlockedEntities::default(),
            groups: GroupMap::default(),
            top_level_groups: Vec::default(),
        }
    }
}

impl<D, E> Configuration<D, E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prefix<I>(mut self, prefix: I) -> Self
    where
        I: Into<String>,
    {
        self.prefixes.push(prefix.into());
        self
    }

    pub fn prefixes<I>(mut self, prefixes: impl IntoIterator<Item = I>) -> Self
    where
        I: Into<String>,
    {
        self.prefixes = prefixes.into_iter().map(|p| p.into()).collect();
        self
    }

    pub fn owners<I>(mut self, iter: impl IntoIterator<Item = I>) -> Self
    where
        I: Into<UserId>,
    {
        self.owners = iter.into_iter().map(|u| u.into()).collect();
        self
    }

    pub fn case_insensitive(mut self, b: bool) -> Self {
        self.case_insensitive = b;

        if b {
            for prefix in &mut self.prefixes {
                *prefix = prefix.to_lowercase();
            }
        }

        self
    }

    pub fn no_dm_prefix(mut self, b: bool) -> Self {
        self.no_dm_prefix = b;
        self
    }

    pub fn on_mention(mut self, id: Option<UserId>) -> Self {
        self.on_mention = id.map(|id| id.to_string());
        self
    }

    pub fn blocked_channels<I>(mut self, iter: impl IntoIterator<Item = I>) -> Self
    where
        I: Into<ChannelId>,
    {
        self.blocked_entities.channels = iter.into_iter().map(|c| c.into()).collect();
        self
    }

    pub fn blocked_guilds<I>(mut self, iter: impl IntoIterator<Item = I>) -> Self
    where
        I: Into<GuildId>,
    {
        self.blocked_entities.guilds = iter.into_iter().map(|c| c.into()).collect();
        self
    }

    pub fn blocked_users<I>(mut self, iter: impl IntoIterator<Item = I>) -> Self
    where
        I: Into<UserId>,
    {
        self.blocked_entities.users = iter.into_iter().map(|c| c.into()).collect();
        self
    }

    pub fn blocked_commands(mut self, iter: impl IntoIterator<Item = CommandId>) -> Self {
        self.blocked_entities.commands = iter.into_iter().collect();
        self
    }

    pub fn blocked_groups(mut self, iter: impl IntoIterator<Item = GroupId>) -> Self {
        self.blocked_entities.groups = iter.into_iter().collect();
        self
    }

    pub fn group(mut self, group: GroupConstructor<D, E>) -> Self {
        let id = GroupId::from(group);

        let group = group();

        if group.prefixes.is_empty() {
            assert!(
                group.subgroups.is_empty(),
                "top level groups must not have prefixes nor subgroups"
            );

            self.top_level_groups.push(group);
            return self;
        }

        for prefix in &group.prefixes {
            self.groups.insert_name(prefix.clone(), id);
        }

        self.groups.insert(id, group);

        self
    }
}
