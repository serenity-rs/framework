use serenity::prelude::{Context as SerenityContext, RwLock};
use serenity::model::prelude::Message;

use std::sync::Arc;
use std::error::Error as StdError;
use std::marker::PhantomData;

pub mod utils;
pub mod context;
pub mod command;
pub mod group;

use utils::IdMap;
use context::Context;
use group::{GroupId, Group, GroupConstructor};

pub type DefaultData = ();
pub type DefaultError = Box<dyn StdError + Send + Sync>;


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
    pub groups: IdMap<String, GroupId, Group<D, E>>,
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
            groups: IdMap::default(),
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
            self.groups.insert_name(prefix.clone(), id);
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

        let command = if let Some(group) = self.groups.get_by_name(prefix_or_name) {
            // TODO: Subgroups

            let name = stream.take_until(|b| b == b' ');

            stream.take_while_char(char::is_whitespace);

            match group.commands.get_by_name(name) {
                Some(command) => command,
                None => return Err(()),
            }
        } else {
            let mut iter = self.groups_without_prefixes.iter();
            loop {
                let group = match iter.next() {
                    Some(group) => group,
                    None => return Err(()),
                };

                if let Some(command) = group.commands.get_by_name(prefix_or_name) {
                    break command;
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
    use crate::{Framework, DefaultError};
    use crate::utils::IdMap;
    use crate::context::Context;
    use crate::command::{Command, CommandResult};
    use crate::group::Group;

    use serenity::model::channel::Message;
    use serenity::futures::future::{BoxFuture, FutureExt};

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
            subcommands: IdMap::new(),
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
