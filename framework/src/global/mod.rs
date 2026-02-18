use std::{collections::HashMap, time::Duration};

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
use utils::{Pointer, error};

pub struct Commands;
impl TypeMapKey for Commands {
    type Value = CommandManager;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserGlobalType {
    Guild(GuildId, UserId),
    User(UserId),
}

#[derive(Debug)]
pub struct GlobalCache<V>(Cache<UserGlobalType, Pointer<V>>)
where
    V: Send + Sync + 'static;

impl<V> Clone for GlobalCache<V>
where
    V: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V> Default for GlobalCache<V>
where
    V: Send + Sync + 'static,
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

impl<V> GlobalCache<V>
where
    V: Send + Sync + 'static,
{
    pub async fn get(&self, guild_id: GuildId, user_id: UserId) -> Option<Pointer<V>> {
        match self.0.get(&UserGlobalType::Guild(guild_id, user_id)).await {
            Some(value) => Some(value),
            None => self.0.get(&UserGlobalType::User(user_id)).await,
        }
    }

    pub async fn insert(&self, key: UserGlobalType, value: V) -> Pointer<V> {
        let ptr: Pointer<V> = Pointer::new(value);
        self.0.insert(key, ptr.clone()).await;
        ptr
    }
}

impl<V> TypeMapKey for GlobalCache<V>
where
    V: Send + Sync + 'static,
{
    type Value = GlobalCache<V>;
}

#[async_trait]
impl<T, V> Extractor<T> for GlobalCache<V>
where
    V: Send + Sync + 'static,
{
    async fn extract(
        ctx: &serenity::all::Context,
        _: &T,
        _: &utils::Pointer<utils::Parser>,
    ) -> Option<Self> {
        GlobalCache::<V>::extract_context(ctx).await
    }
}

#[async_trait]
impl<V> ContextExtractor for GlobalCache<V>
where
    V: Send + Sync + 'static,
{
    async fn extract_context(ctx: &serenity::all::Context) -> Option<Self> {
        let data = ctx.data.read().await;
        data.get::<GlobalCache<V>>().cloned()
    }
}

#[derive(Debug)]
pub struct GlobalMap<V>(Pointer<HashMap<UserGlobalType, Pointer<V>>>)
where
    V: Send + Sync + 'static;

impl<V> TypeMapKey for GlobalMap<V>
where
    V: Send + Sync + 'static,
{
    type Value = GlobalMap<V>;
}

impl<V> Clone for GlobalMap<V>
where
    V: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V> GlobalMap<V>
where
    V: Send + Sync + 'static,
{
    pub async fn get(&self, guild_id: GuildId, user_id: UserId) -> Option<Pointer<V>> {
        let map = self.0.read().await;
        match map.get(&UserGlobalType::Guild(guild_id, user_id)) {
            Some(value) => Some(value.clone()),
            None => map.get(&UserGlobalType::User(user_id)).cloned(),
        }
    }

    pub async fn insert(&self, key: UserGlobalType, value: V) -> Pointer<V> {
        let mut map = self.0.write().await;
        let ptr = Pointer::new(value);
        map.insert(key, ptr.clone());
        ptr
    }

    pub async fn remove(&self, guild_id: GuildId, user_id: UserId) -> Option<V> {
        let mut map = self.0.write().await;
        let ptr = match map.remove(&UserGlobalType::Guild(guild_id, user_id)) {
            Some(value) => value,
            None => map.remove(&UserGlobalType::User(user_id))?,
        };

        match ptr.inner() {
            Ok(value) => Some(value),
            Err(err) => {
                error!("Failed to remove value from GlobalMap: {}", err);
                None
            }
        }
    }
}

#[async_trait]
impl<T, V> Extractor<T> for GlobalMap<V>
where
    V: Send + Sync + 'static,
{
    async fn extract(
        ctx: &serenity::all::Context,
        _: &T,
        _: &utils::Pointer<utils::Parser>,
    ) -> Option<Self> {
        GlobalMap::<V>::extract_context(ctx).await
    }
}

#[async_trait]
impl<V> ContextExtractor for GlobalMap<V>
where
    V: Send + Sync + 'static,
{
    async fn extract_context(ctx: &serenity::all::Context) -> Option<Self> {
        let data = ctx.data.read().await;
        data.get::<GlobalMap<V>>().cloned()
    }
}

impl<T> Default for GlobalMap<T>
where
    T: Send + Sync + 'static,
{
    fn default() -> Self {
        Self(Pointer::new(HashMap::new()))
    }
}
