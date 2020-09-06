//! The official command framework for [Serenity] bots.
//!
//! The framework provides an interface between functionality of the bot and
//! a user on Discord through the concept of *commands*. They are functions
//! that the user invokes in a guild channel or private channel.
//!
//! Command invocations start with a *prefix* at the beginning of the message.
//! The prefix distinguishes normal messages and command invocations. If the prefix
//! is unique, it also avoids collision with command invocations of other bots.
//! The bot may have many prefixes, statically or dynamically defined.
//!
//! Assuming the prefix is `!` and a command with the name `ping` exists, a typical
//! invocation might look like:
//!
//! ```ignore
//! !ping
//! ```
//!
//! Commands can accept arguments. These are the content of the message after
//! the command name. As an example:
//!
//! ```ignore
//! !sort 4 2 8 -3
//! ```
//!
//! The arguments of the `sort` command is a `"4 2 8 -3"` string. Arguments are
//! not processed by the framework, as it is the responsibility of each command
//! to decide the correct format of its arguments, and how they should be parsed.
//!
//! All commands have be assigned to *groups*. Groups are a collection of commands,
//! which most likely share a theme, such as moderation. Groups may
//! participate in the invocation on one of its commands if they have *prefixes*
//! (not to confuse with invocation prefixes). If a group has prefixes, they must be
//! present in the command invocation. Assumming `mod` is a prefix, and `kick` is a
//! command of the group:
//!
//! ```ignore
//! !mod kick @clyde
//! ```
//!
//! Groups without prefixes are called Top Level Groups, as they can only appear once
//! at the beginning of the message implicitly. They are transparent, or "invisible"
//! to the user on Discord.
//!
//! [Serenity]: https://github.com/serenity-rs/serenity

#![warn(missing_docs)]

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

/// The default type for [user data][data] when it is unspecified.
///
/// [data]: struct.Framework.html#structfield.data
pub type DefaultData = ();

/// The default type for [command errors][errors] when it is unspecified.
///
/// [errors]: command/type.CommandResult.html
pub type DefaultError = Box<dyn StdError + Send + Sync>;

/// The core of the framework.
#[derive(Clone)]
pub struct Framework<D = DefaultData, E = DefaultError> {
    /// Configuration of the framework that dictates its behaviour.
    pub conf: Arc<Mutex<Configuration<D, E>>>,
    /// User data that is accessable in every command and function hook.
    pub data: Arc<RwLock<D>>,
}

impl<D, E> Framework<D, E>
where
    D: Default,
{
    /// Creates a new instanstiation of the framework using a given configuration.
    ///
    /// The [`data`] field is [`Default`] initialized.
    ///
    /// [`data`]: #structfield.data
    /// [`Default`]: std::default::Default
    #[inline]
    pub fn new(conf: Configuration<D, E>) -> Self {
        Self::with_data(conf, D::default())
    }
}

impl<D, E> Framework<D, E> {
    /// Creates new instanstiation of the framework using a given configuration and data.
    ///
    /// # Notes
    ///
    /// This consumes the data.
    ///
    /// If you need to retain ownership of the data, consider using [`with_arc_data`].
    ///
    /// [`with_arc_data`]: #method.with_arc_data
    #[inline]
    pub fn with_data(conf: Configuration<D, E>, data: D) -> Self {
        Self::with_arc_data(conf, Arc::new(RwLock::new(data)))
    }

    /// Creates new instanstiation of the framework using a given configuration and data.
    #[inline]
    pub fn with_arc_data(conf: Configuration<D, E>, data: Arc<RwLock<D>>) -> Self {
        Self {
            conf: Arc::new(Mutex::new(conf)),
            data,
        }
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
