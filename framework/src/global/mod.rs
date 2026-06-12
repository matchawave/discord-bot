use std::{collections::HashMap, sync::Arc, time::Duration};

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
    type Value = Arc<CommandManager>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserGlobalType {
    Guild(GuildId, UserId),
    User(UserId),
}

#[derive(Debug)]
pub struct GlobalCache<V>(Cache<UserGlobalType, Option<Pointer<V>>>)
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
    pub async fn get(
        &self,
        guild_id: Option<GuildId>,
        user_id: UserId,
    ) -> Option<Option<Pointer<V>>> {
        if let Some(guild_id) = guild_id
            && let Some(value) = self.0.get(&UserGlobalType::Guild(guild_id, user_id)).await
        {
            return Some(value);
        }
        self.0.get(&UserGlobalType::User(user_id)).await
    }

    pub async fn insert(&self, guild_id: Option<GuildId>, user_id: UserId, value: V) -> Pointer<V> {
        let key = guild_id
            .map(|g_id| UserGlobalType::Guild(g_id, user_id))
            .unwrap_or(UserGlobalType::User(user_id));
        let ptr: Pointer<V> = Pointer::new(value);
        self.0.insert(key, Some(ptr.clone())).await;
        ptr
    }

    pub async fn insert_none(&self, guild_id: Option<GuildId>, user_id: UserId) {
        let key = guild_id
            .map(|g_id| UserGlobalType::Guild(g_id, user_id))
            .unwrap_or(UserGlobalType::User(user_id));
        self.0.insert(key, None).await;
    }

    pub async fn invalidate(&self, guild_id: Option<GuildId>, user_id: UserId) {
        if let Some(guild_id) = guild_id {
            return (self.0)
                .invalidate(&UserGlobalType::Guild(guild_id, user_id))
                .await;
        }
        self.0.invalidate(&UserGlobalType::User(user_id)).await;
    }

    pub async fn invalidate_all(&self) {
        self.0.invalidate_all();
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
pub struct GlobalMap<V>(HashMap<UserGlobalType, Pointer<V>>)
where
    V: Send + Sync + 'static;

impl<V> GlobalMap<V>
where
    V: Send + Sync + 'static,
{
    pub fn get(&self, guild_id: Option<GuildId>, user_id: UserId) -> Option<Pointer<V>> {
        if let Some(guild_id) = guild_id {
            return self
                .0
                .get(&UserGlobalType::Guild(guild_id, user_id))
                .or_else(|| self.0.get(&UserGlobalType::User(user_id)))
                .cloned();
        }
        self.0.get(&UserGlobalType::User(user_id)).cloned()
    }

    pub fn insert(&mut self, guild_id: Option<GuildId>, user_id: UserId, value: V) -> Pointer<V> {
        let key = guild_id
            .map(|g_id| UserGlobalType::Guild(g_id, user_id))
            .unwrap_or(UserGlobalType::User(user_id));
        let ptr = Pointer::new(value);
        self.0.insert(key, ptr.clone());
        ptr
    }

    pub fn remove(&mut self, guild_id: Option<GuildId>, user_id: UserId) -> Option<V> {
        let ptr = match guild_id {
            Some(guild_id) => (self.0)
                .remove(&UserGlobalType::Guild(guild_id, user_id))
                .or_else(|| self.0.remove(&UserGlobalType::User(user_id))),
            None => self.0.remove(&UserGlobalType::User(user_id)),
        }?;

        match ptr.inner() {
            Ok(value) => Some(value),
            Err(err) => {
                error!("Failed to remove value from GlobalMap: {}", err);
                None
            }
        }
    }

    pub fn contains_user(&self, user_id: UserId) -> bool {
        self.0.keys().any(|key| match key {
            UserGlobalType::Guild(_, uid) | UserGlobalType::User(uid) => *uid == user_id,
        })
    }

    pub fn clear_user(&mut self, user_id: UserId) {
        self.0.retain(|key, _| match key {
            UserGlobalType::Guild(_, uid) | UserGlobalType::User(uid) => *uid != user_id,
        });
    }
}

// #[async_trait]
// impl<T, V> Extractor<T> for GlobalMap<V>
// where
//     V: Send + Sync + 'static,
// {
//     async fn extract(
//         ctx: &serenity::all::Context,
//         _: &T,
//         _: &utils::Pointer<utils::Parser>,
//     ) -> Option<Self> {
//         GlobalMap::<V>::extract_context(ctx).await
//     }
// }

// #[async_trait]
// impl<V> ContextExtractor for GlobalMap<V>
// where
//     V: Send + Sync + 'static,
// {
//     async fn extract_context(ctx: &serenity::all::Context) -> Option<Self> {
//         let data = ctx.data.read().await;
//         data.get::<GlobalMap<V>>().cloned()
//     }
// }

impl<T> Default for GlobalMap<T>
where
    T: Send + Sync + 'static,
{
    fn default() -> Self {
        Self(HashMap::new())
    }
}
