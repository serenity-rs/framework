use serenity::model::channel::Message;
use serenity::prelude::{Context as SerenityContext, RwLock, Mutex};

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

pub type DefaultData = ();
pub type DefaultError = Box<dyn StdError + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Framework<D = DefaultData, E = DefaultError> {
    pub conf: Arc<Mutex<Configuration<D, E>>>,
    pub data: Arc<RwLock<D>>,
}

impl<D, E> Framework<D, E>
where
    D: Default
{
    pub fn new(conf: Configuration<D, E>) -> Self {
        Self::with_data(conf, D::default())
    }
}

impl<D, E> Framework<D, E> {
    pub fn with_arc_data(conf: Configuration<D, E>, data: Arc<RwLock<D>>) -> Self {
        Self {
            conf: Arc::new(Mutex::new(conf)),
            data,
        }
    }

    pub fn with_data(conf: Configuration<D, E>, data: D) -> Self {
        Self::with_arc_data(conf, Arc::new(RwLock::new(data)))
    }

    pub async fn dispatch(&self, ctx: SerenityContext, msg: Message) -> Result<(), ()> {
        let command = {
            let conf = self.conf.lock().await;

            let content = if msg.is_private() && conf.no_dm_prefix {
                &msg.content
            } else {
                parse::prefix(&conf, &msg.content).ok_or(())?
            };

            let mut segments = parse::segments(content, ' ');

            let command_name = *segments.peek().ok_or(())?;

            let group = parse::groups(&conf.groups, &mut segments).last();

            let command = match group {
                Some(group) => group.commands.get_by_name(segments.next().ok_or(())?),
                None => {
                    segments.next();

                    conf
                        .top_level_groups
                        .iter()
                        .find_map(|g| g.commands.get_by_name(command_name))
                },
            };

            let mut command = command.ok_or(())?;

            for c in parse::commands(&command.subcommands, &mut segments) {
                command = c;
            }

            command.function
        };

        let ctx = Context {
            data: Arc::clone(&self.data),
            conf: Arc::clone(&self.conf),
            serenity_ctx: ctx,
        };

        assert!(command(ctx, msg).await.is_ok());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::command::{CommandMap, Command, CommandResult};
    use crate::context::Context;
    use crate::group::Group;
    use crate::configuration::Configuration;
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
            subcommands: CommandMap::new(),
        }
    }

    fn general() -> Group<TestData> {
        Group::builder().name("general").command(_ping).build()
    }

    #[tokio::test]
    async fn construction() {
        let _framework: Framework = Framework::new(Configuration::new());
        let _framework: Framework<(), DefaultError> = Framework::new(Configuration::new());
        let _framework: Framework<TestData> = Framework::new(Configuration::new());

        let conf = Configuration::new()
            .group(general);

        let _framework: Framework<TestData> = Framework::with_data(conf, TestData {
            text: "42 is the answer to life, the universe, and everything.".to_string(),
        });
    }
}
