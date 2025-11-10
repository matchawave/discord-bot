use std::sync::Arc;

use serenity::{
    all::{Context, Event, ShardManager},
    async_trait,
    prelude::TypeMapKey,
};
use utils::{Parser, Pointer};

use crate::{command::CommandAction, extractors::Extractor};

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

#[async_trait]
impl Extractor<Event> for Arc<ShardManager> {
    async fn extract(ctx: &Context, _ev: &Event, _p: &Pointer<Parser>) -> Option<Self> {
        ctx.data
            .read()
            .await
            .get::<ShardManagerContainer>()
            .cloned()
    }
}

#[async_trait]
impl Extractor<CommandAction> for Arc<ShardManager> {
    async fn extract(ctx: &Context, _ev: &CommandAction, _p: &Pointer<Parser>) -> Option<Self> {
        ctx.data
            .read()
            .await
            .get::<ShardManagerContainer>()
            .cloned()
    }
}
