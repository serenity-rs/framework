use serenity::model::channel::Message;
use serenity::prelude::{Context as SerenityContext, RwLock};

use std::error::Error as StdError;
use std::sync::Arc;

pub mod command;
pub mod configuration;
pub mod context;
pub mod group;
pub mod parse;
pub mod utils;

use configuration::Configuration;
use context::Context;
use group::{Group, GroupConstructor, GroupId};
use utils::IdMap;

pub type DefaultData = ();
pub type DefaultError = Box<dyn StdError + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Framework<D = DefaultData, E = DefaultError> {
    pub conf: Configuration,
    pub data: Arc<RwLock<D>>,
    pub groups: IdMap<String, GroupId, Group<D, E>>,
    pub top_level_groups: Vec<Group<D, E>>,
}

impl<D, E> Default for Framework<D, E>
where
    D: Default,
{
    fn default() -> Self {
        Self::with_data(D::default())
    }
}

impl<D, E> Framework<D, E>
where
    D: Default,
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
            top_level_groups: Vec::default(),
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

    pub async fn dispatch(&self, ctx: SerenityContext, msg: Message) -> Result<(), ()> {
        // Check for the presence of the prefix.
        if !msg.content.starts_with(&self.conf.prefix) {
            return Err(());
        }

        let content = &msg.content[self.conf.prefix.len()..];
        let mut segments = parse::segments(content, ' ');

        let command_name = *segments.peek().ok_or(())?;

        let group = parse::groups(&self.groups, &mut segments).last();

        let command = match group {
            Some(group) => parse::commands(&group.commands, &mut segments).last(),
            None => self
                .top_level_groups
                .iter()
                .find_map(|g| g.commands.get_by_name(command_name)),
        };

        let command = command.ok_or(())?;

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
    use crate::command::{Command, CommandResult};
    use crate::context::Context;
    use crate::group::Group;
    use crate::utils::IdMap;
    use crate::{DefaultError, Framework};

    use serenity::futures::future::{BoxFuture, FutureExt};
    use serenity::model::channel::Message;

    #[derive(Default)]
    struct TestData {
        text: String,
    }

    fn ping(ctx: Context<TestData>, _msg: Message) -> BoxFuture<'static, CommandResult> {
        async move {
            println!("{:?}", ctx.data.read().await.text);
            Ok(())
        }
        .boxed()
    }

    fn _ping() -> Command<TestData> {
        Command {
            function: ping,
            names: vec!["ping".to_string()],
            subcommands: IdMap::new(),
        }
    }

    fn general() -> Group<TestData> {
        Group::builder().name("general").command(_ping).build()
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
