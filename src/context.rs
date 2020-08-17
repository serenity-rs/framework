use crate::DefaultData;

use serenity::client::Context as SerenityContext;
use serenity::http::{Http, CacheHttp};
use serenity::cache::Cache;
use serenity::prelude::RwLock;


use std::sync::Arc;

#[derive(Clone)]
pub struct Context<D = DefaultData> {
    pub data: Arc<RwLock<D>>,
    pub(crate) parent_ctx: SerenityContext,
}

impl<D> AsRef<Http> for Context<D> {
    fn as_ref(&self) -> &Http {
        &self.parent_ctx.http
    }
}

impl<D> AsRef<Cache> for Context<D> {
    fn as_ref(&self) -> &Cache {
        &self.parent_ctx.cache
    }
}

impl<D> CacheHttp for Context<D>
where
    D: Send + Sync,
{
    fn http(&self) -> &Http {
        &self.parent_ctx.http
    }

    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.parent_ctx.cache)
    }
}
