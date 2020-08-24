use crate::command::CommandId;
use crate::configuration::Configuration;
use crate::group::GroupId;
use crate::{DefaultData, DefaultError};

use serenity::cache::Cache;
use serenity::client::Context as SerenityContext;
use serenity::http::{CacheHttp, Http};
use serenity::prelude::{Mutex, RwLock};

use std::sync::Arc;

#[derive(Clone)]
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

#[derive(Clone)]
pub struct PrefixContext<'a, D = DefaultData, E = DefaultError> {
    pub data: Arc<RwLock<D>>,
    pub conf: &'a Configuration<D, E>,
    pub serenity_ctx: &'a SerenityContext,
}
