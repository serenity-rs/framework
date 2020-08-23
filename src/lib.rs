use serenity::model::channel::Message;
use serenity::prelude::{Context as SerenityContext, Mutex, RwLock};

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
    D: Default,
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

    pub async fn dispatch(&self, ctx: SerenityContext, msg: Message) -> Result<(), ()>
    where
        E: std::fmt::Display,
    {
        let (group_id, command_id, func, args) = {
            let conf = self.conf.lock().await;

            let content = parse::content(&conf, &msg).ok_or(())?;
            let mut segments = parse::Segments::new(&content, ' ', conf.case_insensitive);

            let mut name = segments.next().ok_or(())?;
            let mut group = conf.groups.get_by_name(&*name);

            while let Some(g) = group {
                name = segments.next().ok_or(())?;

                // Check whether there's a subgroup.
                // Only assign it to `group` if it's a part of `group`'s subgroups.
                if let Some((id, aggr)) = conf.groups.get_pair(&*name) {
                    if g.subgroups.contains(&id) {
                        group = Some(aggr);
                        continue;
                    }
                }

                // No more subgroups to be found.
                break;
            }

            // If we could not find more subgroups, `name` will be the segment
            // after the `group`. If we could not find a group itself, `name`
            // will be the segment after the prefix.
            let mut command = conf.commands.get_by_name(&*name).ok_or(())?;

            let group = match group {
                Some(group) if group.commands.contains(&command.id) => group,
                Some(_) => return Err(()),
                None => conf
                    .top_level_groups
                    .iter()
                    .find(|g| g.commands.contains(&command.id))
                    .ok_or(())?,
            };

            // Regardless whether we found a group (and its subgroups) or not,
            // `args` will be a substring of the message after the command.
            let mut args = segments.src;

            while let Some(name) = segments.next() {
                if let Some((id, aggr)) = conf.commands.get_pair(&*name) {
                    if command.subcommands.contains(&id) {
                        command = aggr;
                        args = segments.src;
                        continue;
                    }
                }

                break;
            }

            (group.id, command.id, command.function, args.to_string())
        };

        let ctx = Context {
            data: Arc::clone(&self.data),
            conf: Arc::clone(&self.conf),
            serenity_ctx: ctx,
            group_id,
            command_id,
            args,
        };

        if let Err(err) = func(ctx, msg).await {
            eprintln!("error executing command: {}", err);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::command::{Command, CommandResult};
    use crate::configuration::Configuration;
    use crate::context::Context;
    use crate::group::Group;
    use crate::{DefaultError, Framework};

    use serenity::futures::future::{BoxFuture, FutureExt};
    use serenity::model::channel::Message;

    #[derive(Default)]
    struct TestData {
        text: String,
    }

    fn _ping(ctx: Context<TestData>, _msg: Message) -> BoxFuture<'static, CommandResult> {
        async move {
            println!("{:?}", ctx.data.read().await.text);
            Ok(())
        }
        .boxed()
    }

    fn ping() -> Command<TestData> {
        Command::builder("ping").function(_ping).build()
    }

    fn general() -> Group {
        Group::builder("general").command(ping).build()
    }

    #[tokio::test]
    async fn construction() {
        let _framework: Framework = Framework::new(Configuration::new());
        let _framework: Framework<(), DefaultError> = Framework::new(Configuration::new());
        let _framework: Framework<TestData> = Framework::new(Configuration::new());

        let mut conf = Configuration::new();
        conf.group(general);

        let _framework: Framework<TestData> = Framework::with_data(
            conf,
            TestData {
                text: "42 is the answer to life, the universe, and everything.".to_string(),
            },
        );
    }
}
