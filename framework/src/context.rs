//! Data provided in different *contexts*.
//!
//! A context type contains data that is only available in certain phases of command
//! dispatch. Other than [the Discord message][msg], data is placed into the context
//! types in order to arrange them together and to allow extending the types in future
//! releases without breaking function definitions.
//!
//! [msg]: serenity::model::channel::Message

use crate::command::CommandId;
use crate::configuration::Configuration;
use crate::{DefaultData, DefaultError};

use serenity::cache::Cache;
use serenity::client::Context as SerenityContext;
use serenity::http::{CacheHttp, Http};
use serenity::prelude::{Mutex, RwLock};

use std::sync::Arc;

/// The final context type.
///
/// [Ownership of this context is given to the consumer of the framework][ctx],
/// as it's created in the last phase of the dispatch process: Invoking
/// the command function. Consequently, this context type contains
/// every data that's relevant to the command.
///
/// [ctx]: crate::command::CommandFn
#[non_exhaustive]
pub struct Context<D = DefaultData, E = DefaultError> {
    /// User data.
    pub data: Arc<RwLock<D>>,
    /// Framework configuration.
    pub conf: Arc<Mutex<Configuration<D, E>>>,
    /// Serenity's context type.
    pub serenity_ctx: SerenityContext,
    /// The identifier of the command.
    pub command_id: CommandId,
    /// The [prefix] that was used to invoke this command.
    ///
    /// [prefix]: crate::parse::prefix::content
    pub prefix: String,
    /// The arguments of the command.
    ///
    /// This is the content of the message after the command.
    pub args: String,
}

impl<D, E> Clone for Context<D, E> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            conf: Arc::clone(&self.conf),
            serenity_ctx: self.serenity_ctx.clone(),
            command_id: self.command_id,
            prefix: self.prefix.clone(),
            args: self.args.clone(),
        }
    }
}

impl<D, E> AsRef<Http> for Context<D, E> {
    fn as_ref(&self) -> &Http {
        &self.serenity_ctx.http
    }
}

impl<D, E> AsRef<Cache> for Context<D, E> {
    fn as_ref(&self) -> &Cache {
        &self.serenity_ctx.cache
    }
}

impl<D, E> CacheHttp for Context<D, E>
where
    D: Send + Sync,
    E: Send + Sync,
{
    fn http(&self) -> &Http {
        &self.serenity_ctx.http
    }

    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.serenity_ctx.cache)
    }
}

/// The prefix context.
///
/// This is passed in the [dynamic prefix][dyn_prefix] hook.
///
/// [dyn_prefix]: crate::configuration::DynamicPrefix
#[non_exhaustive]
pub struct PrefixContext<'a, D = DefaultData, E = DefaultError> {
    /// User data.
    pub data: &'a Arc<RwLock<D>>,
    /// Framework configuration.
    pub conf: &'a Configuration<D, E>,
    /// Serenity's context type.
    pub serenity_ctx: &'a SerenityContext,
}

impl<'a, D, E> Clone for PrefixContext<'a, D, E> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            conf: self.conf,
            serenity_ctx: self.serenity_ctx,
        }
    }
}

impl<D, E> AsRef<Http> for PrefixContext<'_, D, E> {
    fn as_ref(&self) -> &Http {
        &self.serenity_ctx.http
    }
}

impl<D, E> AsRef<Cache> for PrefixContext<'_, D, E> {
    fn as_ref(&self) -> &Cache {
        &self.serenity_ctx.cache
    }
}

impl<D, E> CacheHttp for PrefixContext<'_, D, E>
where
    D: Send + Sync,
    E: Send + Sync,
{
    fn http(&self) -> &Http {
        &self.serenity_ctx.http
    }

    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.serenity_ctx.cache)
    }
}

/// The check context.
///
/// This is passed to the [check function][fn].
///
/// [fn]: crate::check::CheckFn
#[non_exhaustive]
pub struct CheckContext<'a, D = DefaultData, E = DefaultError> {
    /// User data.
    pub data: &'a Arc<RwLock<D>>,
    /// Framework configuration.
    pub conf: &'a Configuration<D, E>,
    /// Serenity's context type.
    pub serenity_ctx: &'a SerenityContext,
    /// The identifier of the command that is being checked upon.
    pub command_id: CommandId,
}

impl<'a, D, E> Clone for CheckContext<'a, D, E> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            conf: self.conf,
            serenity_ctx: self.serenity_ctx,
            command_id: self.command_id,
        }
    }
}

impl<D, E> AsRef<Http> for CheckContext<'_, D, E> {
    fn as_ref(&self) -> &Http {
        &self.serenity_ctx.http
    }
}

impl<D, E> AsRef<Cache> for CheckContext<'_, D, E> {
    fn as_ref(&self) -> &Cache {
        &self.serenity_ctx.cache
    }
}

impl<D, E> CacheHttp for CheckContext<'_, D, E>
where
    D: Send + Sync,
    E: Send + Sync,
{
    fn http(&self) -> &Http {
        &self.serenity_ctx.http
    }

    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.serenity_ctx.cache)
    }
}
