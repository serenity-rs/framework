use crate::command::{CommandConstructor, CommandId};
use crate::utils::IdMap;

use std::collections::HashSet;

pub type GroupMap = IdMap<String, GroupId, Group>;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupId(pub(crate) usize);

impl GroupId {
    pub fn into_usize(self) -> usize {
        self.0
    }
}

pub type GroupConstructor = fn() -> Group;

impl From<GroupConstructor> for GroupId {
    fn from(f: GroupConstructor) -> Self {
        Self(f as usize)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub prefixes: Vec<String>,
    pub commands: HashSet<CommandId>,
    pub subgroups: HashSet<GroupId>,
}

impl Group {
    pub fn builder<I>(name: I) -> GroupBuilder
    where
        I: Into<String>,
    {
        GroupBuilder::new(name)
    }
}

#[derive(Debug, Default, Clone)]
pub struct GroupBuilder {
    inner: Group,
}

impl GroupBuilder {
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
        self.inner.name = name.into();

        self
    }

    pub fn prefix<I>(mut self, prefix: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.prefixes.push(prefix.into());
        self
    }

    pub fn prefixes<I>(mut self, iter: impl IntoIterator<Item = I>) -> Self
    where
        I: Into<String>,
    {
        self.inner.prefixes = iter.into_iter().map(|p| p.into()).collect();
        self
    }

    pub fn command<D, E>(mut self, command: CommandConstructor<D, E>) -> Self {
        self.inner.commands.insert(CommandId::from(command));
        self
    }

    pub fn subgroup(mut self, group: GroupConstructor) -> Self {
        self.inner.subgroups.insert(GroupId::from(group));
        self
    }

    pub fn build(self) -> Group {
        self.inner
    }
}
