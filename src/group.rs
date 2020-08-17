use crate::command::{CommandConstructor, CommandId, CommandMap};
use crate::utils::IdMap;
use crate::{DefaultData, DefaultError};

pub type GroupMap<D = DefaultData, E = DefaultError> = IdMap<String, GroupId, Group<D, E>>;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupId(pub u64);

pub type GroupConstructor<D = DefaultData, E = DefaultError> = fn() -> Group<D, E>;

impl<D, E> From<GroupConstructor<D, E>> for GroupId {
    fn from(f: GroupConstructor<D, E>) -> Self {
        Self(f as u64)
    }
}

#[derive(Debug, Clone)]
pub struct Group<D = DefaultData, E = DefaultError> {
    pub name: String,
    pub prefixes: Vec<String>,
    pub commands: CommandMap<D, E>,
    pub subgroups: GroupMap<D, E>,
}

impl<D, E> Default for Group<D, E> {
    fn default() -> Self {
        Self {
            name: String::default(),
            prefixes: Vec::default(),
            commands: IdMap::default(),
            subgroups: IdMap::default(),
        }
    }
}

impl<D, E> Group<D, E> {
    pub fn builder() -> GroupBuilder<D, E> {
        GroupBuilder::default()
    }
}

#[derive(Debug, Clone)]
pub struct GroupBuilder<D = DefaultData, E = DefaultError> {
    inner: Group<D, E>,
}

impl<D, E> Default for GroupBuilder<D, E> {
    fn default() -> Self {
        Self {
            inner: Group::default(),
        }
    }
}

impl<D, E> GroupBuilder<D, E> {
    pub fn name<I>(mut self, name: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.name = name.into();

        self
    }

    pub fn command(mut self, command: CommandConstructor<D, E>) -> Self {
        let id = CommandId::from(command);

        let command = command();

        for name in &command.names {
            self.inner.commands.insert_name(name.clone(), id);
        }

        self.inner.commands.insert(id, command);

        self
    }

    pub fn subgroup(mut self, group: GroupConstructor<D, E>) -> Self {
        let id = GroupId::from(group);

        let group = group();

        assert!(
            !group.prefixes.is_empty(),
            "subgroups cannot have zero prefixes"
        );

        for prefix in &group.prefixes {
            self.inner.subgroups.insert_name(prefix.clone(), id);
        }

        self.inner.subgroups.insert(id, group);

        self
    }

    pub fn build(self) -> Group<D, E> {
        self.inner
    }
}
