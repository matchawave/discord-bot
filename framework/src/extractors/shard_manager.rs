use std::sync::Arc;

use serenity::{
    all::{Context, ShardManager},
    async_trait,
    prelude::TypeMapKey,
};
use utils::{DataType, Parser, Pointer};

use crate::extractors::{ContextExtractor, Extractor};

pub struct ShardManagerContainer(pub Arc<ShardManager>);

impl ShardManagerContainer {
    pub async fn get(data: &DataType) -> Option<Self> {
        data.read()
            .await
            .get::<ShardManagerContainer>()
            .cloned()
            .map(ShardManagerContainer)
    }
}

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

#[async_trait]
impl ContextExtractor for ShardManagerContainer {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        Self::get(&ctx.data).await
    }
}

#[async_trait]
impl<T> Extractor<T> for ShardManagerContainer
where
    T: Send + Sync,
{
    async fn extract(ctx: &Context, _: &T, _: &Pointer<Parser>) -> Option<Self> {
        Self::extract_context(ctx).await
    }
}
