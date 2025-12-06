use serenity::{
    all::{Context, GuildId},
    async_trait,
    prelude::TypeMapKey,
};

use utils::{Parser, Pointer};

use crate::{
    ShardData,
    extractors::{ContextEventExtractor, ContextExtractor, EventExtractor, Extractor},
    guilds::Guilds,
};

#[derive(Clone)]
pub struct Prefix(pub Pointer<Option<String>>);

pub struct DefaultPrefix(pub String);

impl TypeMapKey for Prefix {
    type Value = Pointer<Option<String>>;
}

#[async_trait]
impl<T> ContextEventExtractor<T> for Prefix
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self> {
        let guild_id = GuildId::extract_event(ev).await?;
        let guilds = Guilds::extract_context(ctx).await?;
        (guilds.get::<Prefix, Option<String>>(guild_id))
            .await
            .map(Self)
    }
}

#[async_trait]
impl<T> Extractor<T> for Prefix
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract(ctx: &Context, ev: &T, _: &Pointer<Parser>) -> Option<Self> {
        Prefix::extract_context_event(ctx, ev).await
    }
}

#[async_trait]
impl ContextExtractor for DefaultPrefix {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        let shard_data = ShardData::get(ctx).await?;
        Some(Self(shard_data.default_prefix.clone()))
    }
}

#[async_trait]
impl<T> Extractor<T> for DefaultPrefix
where
    T: Send + Sync + 'static,
{
    async fn extract(ctx: &Context, _: &T, _: &Pointer<Parser>) -> Option<Self> {
        DefaultPrefix::extract_context(ctx).await
    }
}
