use serenity::prelude::{Context as SerenityContext, RwLock};
use serenity::model::prelude::Message;
use serenity::futures::future::{BoxFuture, FutureExt};

use std::sync::Arc;
use std::error::Error as StdError;
use std::marker::PhantomData;
use std::collections::HashMap;

pub type DefaultData = ();
pub type DefaultError = Box<dyn StdError + Send + Sync>;

#[derive(Clone)]
pub struct Context<D = DefaultData> {
    pub data: Arc<RwLock<D>>,
    pub msg: Message,
    parent_ctx: SerenityContext,
}

pub type CommandResult<E = DefaultError> = std::result::Result<(), E>;
pub type CommandFn<D = DefaultData, E = DefaultError> = fn(ctx: Context<D>) -> BoxFuture<'static, CommandResult<E>>;

pub type CommandConstructor<D = DefaultData, E = DefaultError> = fn() -> Command<D, E>;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandId(pub u64);

impl<D, E> From<CommandConstructor<D, E>> for CommandId {
    fn from(f: CommandConstructor<D, E>) -> Self {
        Self(f as u64)
    }
}

#[derive(Debug, Clone)]
pub struct Command<D = DefaultData, E = DefaultError> {
    pub function: CommandFn<D, E>,
    pub names: Vec<String>,
    pub subcommands: Vec<CommandId>,
}

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
    pub command_name_to_id: HashMap<String, CommandId>,
    pub commands: HashMap<CommandId, Command<D, E>>,
    pub subgroup_prefix_to_id: HashMap<String, GroupId>,
    pub subgroups: HashMap<GroupId, Group<D, E>>,
}

impl<D, E> Default for Group<D, E> {
    fn default() -> Self {
        Self {
            name: String::default(),
            prefixes: Vec::default(),
            command_name_to_id: HashMap::default(),
            commands: HashMap::default(),
            subgroup_prefix_to_id: HashMap::default(),
            subgroups: HashMap::default(),
        }
    }
}

impl<D, E> Group<D, E> {
    pub fn builder() -> GroupBuilder<D, E> {
        GroupBuilder {
            inner: Group::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GroupBuilder<D = DefaultData, E = DefaultError> {
    inner: Group<D, E>,
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
            self.inner.command_name_to_id.insert(name.clone(), id);
        }

        self.inner.commands.insert(id, command);

        self
    }

    pub fn subgroup(mut self, group: GroupConstructor<D, E>) -> Self {
        let id = GroupId::from(group);

        let group = group();

        for prefix in &group.prefixes {
            self.inner.subgroup_prefix_to_id.insert(prefix.clone(), id);
        }

        self.inner.subgroups.insert(id, group);

        self
    }

    pub fn build(self) -> Group<D, E> {
        self.inner
    }
}

#[derive(Debug, Clone)]
pub struct Framework<D = DefaultData, E = DefaultError> {
    pub data: Arc<RwLock<D>>,
    pub group_prefix_to_id: HashMap<String, GroupId>,
    pub groups: HashMap<GroupId, Group<D, E>>,
    _error: PhantomData<E>,
}

impl<D, E> Default for Framework<D, E>
where
    D: Default
{
    fn default() -> Self {
        Self::with_data(D::default())
    }
}

impl<D, E> Framework<D, E>
where
    D: Default
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D, E> Framework<D, E> {
    pub fn with_data(data: D) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            group_prefix_to_id: HashMap::default(),
            groups: HashMap::default(),
            _error: PhantomData,
        }
    }

    pub fn group(&mut self, group: GroupConstructor<D, E>) -> &mut Self {
        let id = GroupId::from(group);

        let group = group();

        for prefix in &group.prefixes {
            self.group_prefix_to_id.insert(prefix.clone(), id);
        }

        self.groups.insert(id, group);

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestData {
        text: String,
    }

    fn ping(ctx: Context<TestData>) -> BoxFuture<'static, CommandResult> {
        async move {
            println!("Hello world!");
            println!("{:?}", ctx.data.read().await.text);
            Ok(())
        }.boxed()
    }

    fn _ping() -> Command<TestData> {
        Command {
            function: ping,
            names: vec!["ping".to_string()],
            subcommands: Vec::new(),
        }
    }

    fn general() -> Group<TestData> {
        Group::builder()
            .name("general")
            .command(_ping)
            .build()
    }

    #[tokio::test]
    async fn construction() {
        let _framework: Framework = Framework::new();
        let _framework: Framework<(), DefaultError> = Framework::new();
        let _framework: Framework<TestData> = Framework::new();
        let mut framework: Framework<TestData> = Framework::with_data(TestData {
            text: "42 is the answer to life, the universe, and everything.".to_string(),
        });

        framework.group(general);
    }
}
