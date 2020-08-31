use crate::check::{Check, CheckConstructor};
use crate::command::{CommandConstructor, CommandId};
use crate::utils::IdMap;
use crate::{DefaultData, DefaultError};

use std::collections::HashSet;
use std::fmt;

pub type GroupMap<D = DefaultData, E = DefaultError> = IdMap<String, GroupId, Group<D, E>>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupId(pub(crate) usize);

impl GroupId {
    pub fn into_usize(self) -> usize {
        self.0
    }
}

pub type GroupConstructor<D = DefaultData, E = DefaultError> = fn() -> Group<D, E>;

impl<D, E> From<GroupConstructor<D, E>> for GroupId {
    fn from(f: GroupConstructor<D, E>) -> Self {
        Self(f as usize)
    }
}

#[non_exhaustive]
pub struct Group<D = DefaultData, E = DefaultError> {
    pub id: GroupId,
    pub name: String,
    pub prefixes: Vec<String>,
    pub commands: HashSet<CommandId>,
    pub subgroups: HashSet<GroupId>,
    pub default_command: Option<CommandId>,
    pub description: Option<String>,
    pub checks: Vec<Check<D, E>>,
}

impl<D, E> Clone for Group<D, E> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            prefixes: self.prefixes.clone(),
            commands: self.commands.clone(),
            subgroups: self.subgroups.clone(),
            default_command: self.default_command,
            description: self.description.clone(),
            checks: self.checks.clone(),
        }
    }
}

impl<D, E> Default for Group<D, E> {
    fn default() -> Self {
        Self {
            id: GroupId::from((|| Group::default()) as GroupConstructor<D, E>),
            name: String::default(),
            prefixes: Vec::default(),
            commands: HashSet::default(),
            subgroups: HashSet::default(),
            default_command: None,
            description: None,
            checks: Vec::default(),
        }
    }
}

impl<D, E> fmt::Debug for Group<D, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Group")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("prefixes", &self.prefixes)
            .field("commands", &self.commands)
            .field("subgroups", &self.subgroups)
            .field("default_command", &self.default_command)
            .field("description", &self.description)
            .field("checks", &self.checks)
            .finish()
    }
}

impl<D, E> Group<D, E> {
    pub fn builder<I>(name: I) -> GroupBuilder<D, E>
    where
        I: Into<String>,
    {
        GroupBuilder::new(name)
    }
}

pub struct GroupBuilder<D = DefaultData, E = DefaultError> {
    inner: Group<D, E>,
}

impl<D, E> GroupBuilder<D, E> {
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

    pub fn command(mut self, command: CommandConstructor<D, E>) -> Self {
        self.inner.commands.insert(CommandId::from(command));
        self
    }

    pub fn subgroup(mut self, group: GroupConstructor<D, E>) -> Self {
        self.inner.subgroups.insert(GroupId::from(group));
        self
    }

    pub fn default_command(mut self, command: CommandConstructor<D, E>) -> Self {
        self.inner.default_command = Some(CommandId::from(command));
        self
    }

    pub fn description<I>(mut self, description: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.description = Some(description.into());
        self
    }

    pub fn check(mut self, check: CheckConstructor<D, E>) -> Self {
        self.inner.checks.push(check());
        self
    }

    pub fn build(self) -> Group<D, E> {
        self.inner
    }
}

impl<D, E> Clone for GroupBuilder<D, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<D, E> Default for GroupBuilder<D, E> {
    fn default() -> Self {
        Self {
            inner: Group::default(),
        }
    }
}

impl<D, E> fmt::Debug for GroupBuilder<D, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GroupBuilder")
            .field("inner", &self.inner)
            .finish()
    }
}
