use std::sync::Arc;

use serenity::{all::Context, async_trait};
use utils::{Parser, Pointer};

use crate::extractors::Extractor;

#[async_trait]
impl<T> Extractor<T> for Arc<serenity::http::Http>
where
    T: Send + Sync,
{
    async fn extract(ctx: &Context, _e: &T, _p: &Pointer<Parser>) -> Option<Self> {
        Some(ctx.http.clone())
    }
}
