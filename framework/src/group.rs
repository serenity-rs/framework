//! Functions and types relating to groups.
//!
//! A group is a collection of commands. It may have prefixes that create
//! an association between it and its commands for the user on Discord invoking
//! one of the commands. It may have none, in which case it is regarded as a
//! Top Level Group. It is transparent to the user, and only useful for applying
//! [`check`]s across all of its commands or displaying information in help commands.
//! It may have subgroups to arrange functionality together. If a group has prefixes,
//! it may define a default command. This command is chosen when an invocation only
//! contains one of the group's prefixes.
//!
//! [`check`]: crate::check

use crate::check::{Check, CheckConstructor};
use crate::command::{CommandConstructor, CommandId};
use crate::utils::IdMap;
use crate::{DefaultData, DefaultError};

use std::collections::HashSet;
use std::fmt;

/// [`IdMap`] for storing groups.
///
/// [`IdMap`]: crate::utils::IdMap
pub type GroupMap<D = DefaultData, E = DefaultError> = IdMap<String, GroupId, Group<D, E>>;

/// A constructor of the [`Group`] type provided by the consumer of the framework.
pub type GroupConstructor<D = DefaultData, E = DefaultError> = fn() -> Group<D, E>;

/// A unique identifier of a [`Group`] stored in the [`GroupMap`].
///
/// It is constructed from [`GroupConstructor`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupId(pub(crate) usize);

impl GroupId {
    /// Converts the identifier to its internal representation.
    pub fn into_usize(self) -> usize {
        self.0
    }

    /// Converts the identifier to the constructor it points to.
    pub(crate) fn into_constructor<D, E>(self) -> GroupConstructor<D, E> {
        // SAFETY: GroupId in user code can only be constructed by its
        // `From<GroupConstructor<D, E>>` impl. This makes the transmute safe.

        unsafe { std::mem::transmute(self.0 as *const ()) }
    }
}

impl<D, E> From<GroupConstructor<D, E>> for GroupId {
    fn from(f: GroupConstructor<D, E>) -> Self {
        Self(f as usize)
    }
}

/// Data surrounding a group.
#[non_exhaustive]
pub struct Group<D = DefaultData, E = DefaultError> {
    /// The identifier of this group.
    pub id: GroupId,
    /// The name of this group.
    pub name: String,
    /// The prefixes of this group by which it can be invoked.
    pub prefixes: Vec<String>,
    /// The commands belonging to this group.
    pub commands: HashSet<CommandId>,
    /// A list of subgroups of this group.
    pub subgroups: HashSet<GroupId>,
    /// A default command of this group.
    pub default_command: Option<CommandId>,
    /// A string describing this group.
    pub description: Option<String>,
    /// A function that allows/denies access to this group's commands.
    pub check: Option<Check<D, E>>,
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
            check: self.check.clone(),
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
            check: None,
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
            .field("check", &self.check)
            .finish()
    }
}

impl<D, E> Group<D, E> {
    /// Constructs a builder that will be used to create a group from scratch.
    ///
    /// Argument is the name of the group.
    pub fn builder<I>(name: I) -> GroupBuilder<D, E>
    where
        I: Into<String>,
    {
        GroupBuilder::new(name)
    }
}

/// A builder type for creating a [`Group`] from scratch.
pub struct GroupBuilder<D = DefaultData, E = DefaultError> {
    inner: Group<D, E>,
}

impl<D, E> GroupBuilder<D, E> {
    /// Constructs a new instance of the builder.
    ///
    /// Argument is the name of the group.
    pub fn new<I>(name: I) -> Self
    where
        I: Into<String>,
    {
        Self::default().name(name)
    }

    /// Assing the name of this group.
    pub fn name<I>(mut self, name: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.name = name.into();

        self
    }

    /// Assign a prefix to this group.
    ///
    /// The prefix is added to the [`prefixes`] list.
    ///
    /// [`prefixes`]: Group::prefixes
    pub fn prefix<I>(mut self, prefix: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.prefixes.push(prefix.into());
        self
    }

    /// Assign a command to this group.
    ///
    /// The command is added to the [`commands`] list.
    ///
    /// [`commands`]: Group::commands
    pub fn command(mut self, command: CommandConstructor<D, E>) -> Self {
        self.inner.commands.insert(CommandId::from(command));
        self
    }

    /// Assign a subgroup to this group.
    ///
    /// The subgroup is added to the [`subgroups`] list.
    ///
    /// [`subgroups`]: Group::subgroups
    pub fn subgroup(mut self, group: GroupConstructor<D, E>) -> Self {
        self.inner.subgroups.insert(GroupId::from(group));
        self
    }

    /// Assign a default command to this group.
    pub fn default_command(mut self, command: CommandConstructor<D, E>) -> Self {
        self.inner.default_command = Some(CommandId::from(command));
        self
    }

    /// Assign a description to this group.
    pub fn description<I>(mut self, description: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.description = Some(description.into());
        self
    }

    /// Assign a check to this group.
    pub fn check(mut self, check: CheckConstructor<D, E>) -> Self {
        self.inner.check = Some(check());
        self
    }

    /// Complete building a group.
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
