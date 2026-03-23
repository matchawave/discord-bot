use serenity::{
    all::{Context, User},
    async_trait,
};

use utils::{Parser, Pointer};

use crate::{
    ShardData,
    extractors::{ContextExtractor, Extractor},
};

pub struct Bot(pub User);

#[async_trait]
impl ContextExtractor for Bot {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        let bot = ShardData::get(ctx.shard_id, &ctx.data).await?.bot;
        bot.read().await.clone().map(Bot)
    }
}

#[async_trait]
impl<T> Extractor<T> for Bot
where
    T: Send + Sync,
{
    async fn extract(ctx: &Context, _e: &T, _p: &Pointer<Parser>) -> Option<Self> {
        Self::extract_context(ctx).await
    }
}
