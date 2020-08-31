use serenity::model::channel::Message;
use serenity::prelude::{Context as SerenityContext, Mutex, RwLock};

use std::error::Error as StdError;
use std::future::Future;
use std::sync::Arc;

pub mod check;
pub mod command;
pub mod configuration;
pub mod context;
pub mod error;
pub mod group;
pub mod parse;
pub mod prelude;
pub mod utils;

use command::Command;
use command::{CommandFn, CommandResult};
use configuration::Configuration;
use context::{CheckContext, Context};
use error::{DispatchError, Error};
use group::Group;
use utils::Segments;

pub type DefaultData = ();
pub type DefaultError = Box<dyn StdError + Send + Sync>;

#[derive(Clone)]
pub struct Framework<D = DefaultData, E = DefaultError> {
    pub conf: Arc<Mutex<Configuration<D, E>>>,
    pub data: Arc<RwLock<D>>,
}

impl<D, E> Framework<D, E>
where
    D: Default,
{
    #[inline]
    pub fn new(conf: Configuration<D, E>) -> Self {
        Self::with_data(conf, D::default())
    }
}

impl<D, E> Framework<D, E> {
    #[inline]
    pub fn with_arc_data(conf: Configuration<D, E>, data: Arc<RwLock<D>>) -> Self {
        Self {
            conf: Arc::new(Mutex::new(conf)),
            data,
        }
    }

    #[inline]
    pub fn with_data(conf: Configuration<D, E>, data: D) -> Self {
        Self::with_arc_data(conf, Arc::new(RwLock::new(data)))
    }

    #[inline]
    pub async fn dispatch(&self, ctx: SerenityContext, msg: Message) -> Result<(), Error<E>> {
        self.dispatch_with_hook(ctx, msg, |ctx, msg, f| async move { f(ctx, msg).await })
            .await
    }

    pub async fn dispatch_with_hook<F, Fut>(
        &self,
        ctx: SerenityContext,
        msg: Message,
        hook: F,
    ) -> Result<(), Error<E>>
    where
        F: FnOnce(Context<D, E>, Message, CommandFn<D, E>) -> Fut,
        Fut: Future<Output = CommandResult<(), E>>,
    {
        let (func, group_id, command_id, prefix, args) = {
            let conf = self.conf.lock().await;

            is_blocked(&conf, &msg)?;

            let (prefix, content) = match parse::content(&self.data, &conf, &ctx, &msg).await {
                Some(pair) => pair,
                None => return Err(Error::Dispatch(DispatchError::NormalMessage)),
            };

            if content.is_empty() {
                return Err(Error::Dispatch(DispatchError::PrefixOnly));
            }

            let mut segments = Segments::new(&content, ' ', conf.case_insensitive);

            let group = parse::group(&conf, &mut segments, |group| {
                group_checks(&self.data, &conf, &ctx, &msg, group)
            })
            .await?;

            let (group, command) = parse::command(&conf, &mut segments, group, |group, command| {
                command_checks(&self.data, &conf, &ctx, &msg, group, command)
            })
            .await?;

            let args = segments.source();

            (
                command.function,
                group.id,
                command.id,
                prefix.to_string(),
                args.to_string(),
            )
        };

        let ctx = Context {
            data: Arc::clone(&self.data),
            conf: Arc::clone(&self.conf),
            serenity_ctx: ctx,
            group_id,
            command_id,
            prefix,
            args,
        };

        hook(ctx, msg, func).await.map_err(Error::User)
    }
}

fn is_blocked<D, E>(conf: &Configuration<D, E>, msg: &Message) -> Result<(), DispatchError> {
    if conf.blocked_entities.users.contains(&msg.author.id) {
        return Err(DispatchError::BlockedUser(msg.author.id));
    }

    if conf.blocked_entities.channels.contains(&msg.channel_id) {
        return Err(DispatchError::BlockedChannel(msg.channel_id));
    }

    if let Some(guild_id) = msg.guild_id {
        if conf.blocked_entities.guilds.contains(&guild_id) {
            return Err(DispatchError::BlockedGuild(guild_id));
        }
    }

    Ok(())
}

async fn group_checks<D, E>(
    data: &Arc<RwLock<D>>,
    conf: &Configuration<D, E>,
    serenity_ctx: &SerenityContext,
    msg: &Message,
    group: &Group<D, E>,
) -> Result<(), Error<E>> {
    let ctx = CheckContext {
        data,
        conf,
        serenity_ctx,
        group_id: group.id,
        command_id: None,
    };

    for check in &group.checks {
        if let Err(reason) = (check.function)(&ctx, msg).await {
            return Err(Error::Dispatch(DispatchError::CheckFailed(
                check.name.clone(),
                reason,
            )));
        }
    }

    Ok(())
}

async fn command_checks<D, E>(
    data: &Arc<RwLock<D>>,
    conf: &Configuration<D, E>,
    serenity_ctx: &SerenityContext,
    msg: &Message,
    group: &Group<D, E>,
    command: &Command<D, E>,
) -> Result<(), Error<E>> {
    let ctx = CheckContext {
        data,
        conf,
        serenity_ctx,
        group_id: group.id,
        command_id: Some(command.id),
    };

    for check in &command.checks {
        if let Err(reason) = (check.function)(&ctx, msg).await {
            return Err(Error::Dispatch(DispatchError::CheckFailed(
                check.name.clone(),
                reason,
            )));
        }
    }

    Ok(())
}
