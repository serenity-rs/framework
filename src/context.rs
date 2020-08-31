use crate::command::CommandId;
use crate::configuration::Configuration;
use crate::group::GroupId;
use crate::{DefaultData, DefaultError};

use serenity::cache::Cache;
use serenity::client::Context as SerenityContext;
use serenity::http::{CacheHttp, Http};
use serenity::prelude::{Mutex, RwLock};

use std::sync::Arc;

#[non_exhaustive]
pub struct Context<D = DefaultData, E = DefaultError> {
    pub data: Arc<RwLock<D>>,
    pub conf: Arc<Mutex<Configuration<D, E>>>,
    pub serenity_ctx: SerenityContext,
    pub group_id: GroupId,
    pub command_id: CommandId,
    pub command_name: String,
    pub prefix: String,
    pub args: String,
}

impl<D, E> Clone for Context<D, E> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            conf: Arc::clone(&self.conf),
            serenity_ctx: self.serenity_ctx.clone(),
            group_id: self.group_id,
            command_id: self.command_id,
            command_name: self.command_name.clone(),
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

#[non_exhaustive]
pub struct PrefixContext<'a, D = DefaultData, E = DefaultError> {
    pub data: &'a Arc<RwLock<D>>,
    pub conf: &'a Configuration<D, E>,
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

#[non_exhaustive]
pub struct CheckContext<'a, D = DefaultData, E = DefaultError> {
    pub data: &'a Arc<RwLock<D>>,
    pub conf: &'a Configuration<D, E>,
    pub serenity_ctx: &'a SerenityContext,
    pub group_id: Option<GroupId>,
    pub command_id: Option<CommandId>,
}

impl<'a, D, E> Clone for CheckContext<'a, D, E> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            conf: self.conf,
            serenity_ctx: self.serenity_ctx,
            group_id: self.group_id,
            command_id: self.command_id,
        }
    }
}
