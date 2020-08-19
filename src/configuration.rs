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
    pub prefix: String,
    pub blocked_entities: BlockedEntities,
    pub groups: GroupMap<D, E>,
    pub top_level_groups: Vec<Group<D, E>>,
}

impl<D, E> Default for Configuration<D, E> {
    fn default() -> Self {
        Self {
            prefix: String::default(),
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
        self.prefix = prefix.into();
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
}
