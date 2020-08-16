use serenity::prelude::{Context as SerenityContext, RwLock};
use serenity::model::prelude::Message;
use serenity::futures::future::{BoxFuture, FutureExt};
use serenity::http::Http;
use serenity::cache::Cache;

use std::sync::Arc;
use std::error::Error as StdError;
use std::marker::PhantomData;
use std::collections::HashMap;

pub type DefaultData = ();
pub type DefaultError = Box<dyn StdError + Send + Sync>;

#[derive(Clone)]
pub struct Context<D = DefaultData> {
    pub data: Arc<RwLock<D>>,
    parent_ctx: SerenityContext,
}

impl<D> Context<D> {
    pub fn http(&self) -> Arc<Http> {
        self.parent_ctx.http.clone()
    }

    pub fn cache(&self) -> Arc<Cache> {
        self.parent_ctx.cache.clone()
    }
}

pub type CommandResult<E = DefaultError> = std::result::Result<(), E>;
pub type CommandFn<D = DefaultData, E = DefaultError> = fn(ctx: Context<D>, msg: Message) -> BoxFuture<'static, CommandResult<E>>;

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

        assert!(!group.prefixes.is_empty(), "subgroups cannot have zero prefixes");

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

#[non_exhaustive]
#[derive(Debug, Default, Clone)]
pub struct Configuration {
    pub prefix: String,
}

impl Configuration {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prefix<I>(mut self, prefix: I) -> Self
    where
        I: Into<String>
    {
        self.prefix = prefix.into();
        self
    }
}

#[derive(Debug, Clone)]
pub struct Framework<D = DefaultData, E = DefaultError> {
    pub conf: Configuration,
    pub data: Arc<RwLock<D>>,
    pub group_prefix_to_id: HashMap<String, GroupId>,
    pub groups: HashMap<GroupId, Group<D, E>>,
    pub groups_without_prefixes: Vec<Group<D, E>>,
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
            conf: Configuration::default(),
            data: Arc::new(RwLock::new(data)),
            group_prefix_to_id: HashMap::default(),
            groups: HashMap::default(),
            groups_without_prefixes: Vec::default(),
            _error: PhantomData,
        }
    }

    pub fn configuration(&mut self, conf: Configuration) -> &mut Self {
        self.conf = conf;
        self
    }

    pub fn group(&mut self, group: GroupConstructor<D, E>) -> &mut Self {
        let id = GroupId::from(group);

        let group = group();

        if group.prefixes.is_empty() {
            self.groups_without_prefixes.push(group);
            return self;
        }

        for prefix in &group.prefixes {
            self.group_prefix_to_id.insert(prefix.clone(), id);
        }

        self.groups.insert(id, group);

        self
    }

    pub async fn dispatch(&self, ctx: SerenityContext, msg: Message) -> Result<(), ()> {
        let mut stream = uwl::Stream::new(msg.content.trim());

        // Check for the presence of the prefix.
        if stream.advance(self.conf.prefix.len()) != self.conf.prefix {
            return Err(());
        }

        // Prefix is present, but no more information is available.
        if stream.is_empty() {
            return Err(());
        }

        let prefix_or_name = stream.take_until(|b| b == b' ');

        // Ignore whitespace after the group prefix or command name.
        stream.take_while_char(char::is_whitespace);

        let command = if let Some(id) = self.group_prefix_to_id.get(prefix_or_name) {
            let group = &self.groups[id];

            // TODO: Subgroups

            let name = stream.take_until(|b| b == b' ');

            stream.take_while_char(char::is_whitespace);

            let id = match group.command_name_to_id.get(name) {
                Some(id) => id,
                None => return Err(()),
            };

            &group.commands[id]
        } else {
            let mut iter = self.groups_without_prefixes.iter();
            loop {
                let group = match iter.next() {
                    Some(group) => group,
                    None => return Err(()),
                };

                if let Some(id) = group.command_name_to_id.get(prefix_or_name) {
                    break &group.commands[id];
                }
            }
        };

        let ctx = Context {
            data: self.data.clone(),
            parent_ctx: ctx,
        };

        assert!((command.function)(ctx, msg).await.is_ok());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestData {
        text: String,
    }

    fn ping(ctx: Context<TestData>, _msg: Message) -> BoxFuture<'static, CommandResult> {
        async move {
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
