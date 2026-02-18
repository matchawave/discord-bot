pub mod command;
pub mod data;
pub mod event;
pub mod extractors;
pub mod global;
pub mod guilds;
pub mod handler;
pub mod processes;
pub mod websocket;

pub use macros::*;
use utils::{DataType, Pointer};

use std::sync::Arc;

use serenity::{
    all::{Context, ShardId, User},
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};

use crate::{
    extractors::{ContextExtractor, Extractor},
    guilds::Guilds,
};

pub trait Extractable {
    fn init(map: &mut TypeMap);
    fn retrieve(map: &Arc<TypeMap>) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Clone)]
pub struct ShardData {
    pub guilds: Guilds,
    default_prefix: String,
    bot: Pointer<Option<User>>,
}

impl Default for ShardData {
    fn default() -> Self {
        Self {
            guilds: Guilds::default(),
            default_prefix: "!".to_string(),
            bot: Pointer::default(),
        }
    }
}

impl TypeMapKey for ShardData {
    type Value = Pointer<Vec<ShardData>>;
}

impl ShardData {
    pub fn init(shards: usize, data: &mut TypeMap) {
        let mut data_vec = Vec::with_capacity(shards);
        for _ in 0..shards {
            data_vec.push(ShardData::default());
        }
        data.insert::<ShardData>(Pointer::new(data_vec));
    }

    pub async fn get(shard_id: ShardId, data: &DataType) -> Option<ShardData> {
        let data = data.read().await;
        let data = data.get::<ShardData>()?;
        let shard_id = shard_id.get() as usize;

        data.read().await.get(shard_id).cloned()
    }
}

#[async_trait]
impl ContextExtractor for ShardData {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        ShardData::get(ctx.shard_id, &ctx.data).await
    }
}

#[async_trait]
impl<T> Extractor<T> for ShardData
where
    T: Send + Sync + 'static,
{
    async fn extract(ctx: &Context, _: &T, _: &Pointer<utils::Parser>) -> Option<Self> {
        ShardData::extract_context(ctx).await
    }
}
