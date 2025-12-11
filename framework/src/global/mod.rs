use std::time::Duration;

use crate::{
    command::CommandManager,
    extractors::{ContextExtractor, Extractor},
};
use moka::future::Cache;
use serenity::{
    all::{GuildId, UserId},
    async_trait,
    prelude::TypeMapKey,
};
use utils::Pointer;

pub struct Commands;
impl TypeMapKey for Commands {
    type Value = CommandManager;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserGlobalType {
    Guild(GuildId, UserId),
    User(UserId),
}

impl<V> Default for UserConfigHash<V>
where
    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self(
            Cache::builder()
                .max_capacity(100_000)
                .time_to_live(Duration::from_hours(12))
                .build(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct UserConfigHash<V>(Cache<UserGlobalType, Pointer<V>>)
where
    V: Clone + Send + Sync + 'static;

impl<V> UserConfigHash<V>
where
    V: Clone + Send + Sync + 'static,
{
    pub async fn get(&self, guild_id: GuildId, user_id: UserId) -> Option<Pointer<V>> {
        match self.0.get(&UserGlobalType::Guild(guild_id, user_id)).await {
            Some(value) => Some(value),
            None => self.0.get(&UserGlobalType::User(user_id)).await,
        }
    }

    pub async fn get_cloned(&self, guild_id: GuildId, user_id: UserId) -> Option<V> {
        match self.get(guild_id, user_id).await {
            Some(value) => Some(value.make_clone().await),
            None => None,
        }
    }

    pub async fn insert(&self, key: UserGlobalType, value: V) {
        self.0.insert(key, Pointer::new(value)).await;
    }
}

impl<V> TypeMapKey for UserConfigHash<V>
where
    V: Clone + Send + Sync + 'static,
{
    type Value = UserConfigHash<V>;
}

#[async_trait]
impl<T, V> Extractor<T> for UserConfigHash<V>
where
    V: Clone + Send + Sync + 'static,
{
    async fn extract(
        ctx: &serenity::all::Context,
        _: &T,
        _: &utils::Pointer<utils::Parser>,
    ) -> Option<Self> {
        UserConfigHash::<V>::extract_context(ctx).await
    }
}

#[async_trait]
impl<V> ContextExtractor for UserConfigHash<V>
where
    V: Clone + Send + Sync + 'static,
{
    async fn extract_context(ctx: &serenity::all::Context) -> Option<Self> {
        let data = ctx.data.read().await;
        data.get::<UserConfigHash<V>>().cloned()
    }
}
