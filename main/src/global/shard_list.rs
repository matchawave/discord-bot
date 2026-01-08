use std::sync::Arc;

use framework::extractors::{ContextExtractor, Extractor};
use serenity::{
    all::{GuildId, ShardId},
    async_trait,
    prelude::TypeMapKey,
};
use utils::{DataType, Pointer};

pub struct ShardList(Arc<Vec<Pointer<ShardListData>>>);

#[derive(Default, Clone)]
pub struct ShardListData {
    pub servers: Vec<GuildId>, // List of server IDs in this shard
    pub members: u32,          // Total number of members across all servers in this shard
}

impl ShardList {
    pub fn build(shards: usize) -> Vec<Pointer<ShardListData>> {
        let mut list = Vec::with_capacity(shards);
        for _ in 0..shards {
            list.push(ShardListData::default().into());
        }
        list
    }

    pub async fn get_clone(&self, shard_id: ShardId) -> Option<ShardListData> {
        let shard_data = self.0.get(shard_id.get() as usize)?;
        Some(shard_data.make_clone().await)
    }

    pub async fn get_ptr(&self, shard_id: ShardId) -> Option<Pointer<ShardListData>> {
        self.0.get(shard_id.get() as usize).cloned()
    }

    pub async fn from_data(data: &DataType) -> Option<Self> {
        data.read().await.get::<ShardList>().cloned().map(ShardList)
    }
}

impl TypeMapKey for ShardList {
    type Value = Arc<Vec<Pointer<ShardListData>>>;
}

#[async_trait]
impl ContextExtractor for ShardList {
    async fn extract_context(ctx: &serenity::all::Context) -> Option<Self> {
        (ctx.data.read().await)
            .get::<ShardList>()
            .cloned()
            .map(ShardList)
    }
}

#[async_trait]
impl<T> Extractor<T> for ShardList
where
    T: Send + Sync,
{
    async fn extract(
        ctx: &serenity::all::Context,
        _: &T,
        _: &Pointer<utils::Parser>,
    ) -> Option<Self> {
        Self::extract_context(ctx).await
    }
}

pub struct ShardData(Pointer<ShardListData>);

impl ShardData {
    pub async fn add_member(&self, count: u32) {
        let mut data = self.0.write().await;
        data.members += count;
    }

    pub async fn remove_member(&self, count: u32) {
        let mut data = self.0.write().await;
        data.members = data.members.saturating_sub(count);
    }

    pub async fn add_server(&self, server_id: GuildId) {
        let mut data = self.0.write().await;
        data.servers.push(server_id);
    }

    pub async fn remove_server(&self, server_id: GuildId) {
        let mut data = self.0.write().await;
        data.servers.retain(|id| *id != server_id);
    }
}

#[async_trait]
impl ContextExtractor for ShardData {
    async fn extract_context(ctx: &serenity::all::Context) -> Option<Self> {
        let shard_id = ctx.shard_id;
        let list = (ShardList::extract_context(ctx).await)?;
        list.get_ptr(shard_id).await.map(ShardData)
    }
}

#[async_trait]
impl<T> Extractor<T> for ShardData
where
    T: Send + Sync,
{
    async fn extract(
        ctx: &serenity::all::Context,
        _: &T,
        _: &Pointer<utils::Parser>,
    ) -> Option<Self> {
        Self::extract_context(ctx).await
    }
}
