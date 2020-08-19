use crate::{DefaultData, DefaultError};
use crate::configuration::Configuration;

use serenity::cache::Cache;
use serenity::client::Context as SerenityContext;
use serenity::http::{CacheHttp, Http};
use serenity::prelude::{RwLock, Mutex};

use std::sync::Arc;

#[derive(Clone)]
pub struct Context<D = DefaultData, E = DefaultError> {
    pub data: Arc<RwLock<D>>,
    pub conf: Arc<Mutex<Configuration<D, E>>>,
    pub serenity_ctx: SerenityContext,
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
