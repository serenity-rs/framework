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
//! ```text
//! !ping
//! ```
//!
//! Commands can accept arguments. These are the content of the message after
//! the command name. As an example:
//!
//! ```text
//! !sort 4 2 8 -3
//! ```
//!
//! The arguments of the `sort` command is a `"4 2 8 -3"` string. Arguments are
//! not processed by the framework, as it is the responsibility of each command
//! to decide the correct format of its arguments, and how they should be parsed.
//!
//! Commands may be *categorized*. A category is a list of individual commands
//! with a common theme, such as moderation. They do not participate in command
//! invocation. They are used to register commands in bulk and display related
//! commands in the help command.
//!
//! [Serenity]: https://github.com/serenity-rs/serenity

#![warn(missing_docs)]

use serenity::model::channel::Message;
use serenity::prelude::{Context as SerenityContext, Mutex, RwLock};

use std::future::Future;
use std::sync::Arc;

pub mod argument;
pub mod category;
pub mod check;
pub mod command;
pub mod configuration;
pub mod context;
pub mod error;
pub mod parse;
pub mod prelude;
pub mod utils;

use command::Command;
use command::{CommandFn, CommandResult};
use configuration::Configuration;
use context::{CheckContext, Context};
use error::{DispatchError, Error};
use utils::Segments;

/// The default type for [user data][data] when it is unspecified.
///
/// [data]: Framework::data
pub type DefaultData = ();

/// The core of the framework.
#[derive(Clone)]
pub struct Framework<D = DefaultData> {
    /// Configuration of the framework that dictates its behaviour.
    pub conf: Arc<Mutex<Configuration<D>>>,
    /// User data that is accessable in every command and function hook.
    pub data: Arc<RwLock<D>>,
}

impl<D> Framework<D>
where
    D: Default,
{
    /// Creates a new instanstiation of the framework using a given configuration.
    ///
    /// The [`data`] field is [`Default`] initialized.
    ///
    /// [`data`]: Self::data
    /// [`Default`]: std::default::Default
    #[inline]
    pub fn new(conf: Configuration<D>) -> Self {
        Self::with_data(conf, D::default())
    }
}

impl<D> Framework<D> {
    /// Creates new instanstiation of the framework using a given configuration and data.
    ///
    /// # Notes
    ///
    /// This consumes the data.
    ///
    /// If you need to retain ownership of the data, consider using [`with_arc_data`].
    ///
    /// [`with_arc_data`]: Self::with_arc_data
    #[inline]
    pub fn with_data(conf: Configuration<D>, data: D) -> Self {
        Self::with_arc_data(conf, Arc::new(RwLock::new(data)))
    }

    /// Creates new instanstiation of the framework using a given configuration and data.
    #[inline]
    pub fn with_arc_data(conf: Configuration<D>, data: Arc<RwLock<D>>) -> Self {
        Self {
            conf: Arc::new(Mutex::new(conf)),
            data,
        }
    }

    /// Dispatches a command.
    #[inline]
    pub async fn dispatch(&self, ctx: SerenityContext, msg: Message) -> Result<(), Error> {
        self.dispatch_with_hook(ctx, msg, |ctx, msg, f| f(ctx, msg))
            .await
    }

    /// Dispatches a command with a hook.
    pub async fn dispatch_with_hook<F, Fut>(
        &self,
        ctx: SerenityContext,
        msg: Message,
        hook: F,
    ) -> Result<(), Error>
    where
        F: FnOnce(Context<D>, Message, CommandFn<D>) -> Fut,
        Fut: Future<Output = CommandResult<()>>,
    {
        let (func, command_id, prefix, args) = {
            let conf = self.conf.lock().await;

            let (prefix, content) = match parse::content(&self.data, &conf, &ctx, &msg).await {
                Some(pair) => pair,
                None => return Err(Error::Dispatch(DispatchError::NormalMessage)),
            };

            let mut segments = Segments::new(&content, " ", conf.case_insensitive);

            let mut command = None;

            for cmd in parse::commands(&conf, &mut segments) {
                let cmd = cmd?;

                command_check(&self.data, &conf, &ctx, &msg, cmd).await?;

                command = Some(cmd);
            }

            let command = match command {
                Some(cmd) => cmd,
                None =>
                    return Err(Error::Dispatch(DispatchError::PrefixOnly(
                        prefix.to_string(),
                    ))),
            };

            let args = segments.source();

            (
                command.function,
                command.id,
                prefix.to_string(),
                args.to_string(),
            )
        };

        let ctx = Context {
            data: Arc::clone(&self.data),
            conf: Arc::clone(&self.conf),
            serenity_ctx: ctx,
            command_id,
            prefix,
            args,
        };

        hook(ctx, msg, func).await.map_err(Error::User)
    }
}

async fn command_check<D>(
    data: &Arc<RwLock<D>>,
    conf: &Configuration<D>,
    serenity_ctx: &SerenityContext,
    msg: &Message,
    command: &Command<D>,
) -> Result<(), Error> {
    let ctx = CheckContext {
        data,
        conf,
        serenity_ctx,
        command_id: command.id,
    };

    if let Some(check) = &command.check {
        if let Err(reason) = (check.function)(&ctx, msg).await {
            return Err(Error::Dispatch(DispatchError::CheckFailed(
                check.name.clone(),
                reason,
            )));
        }
    }

    Ok(())
}
