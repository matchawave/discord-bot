use std::time::Duration;

use crate::{GlobalExtractable, command::CommandManager, extractors::Extractor};
use moka::future::Cache;
use serenity::{
    all::{GuildId, UserId},
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};
use utils::{DataType, Pointer};

#[macro_export]
macro_rules! sharded_data {
    ($struct_name: ident, $key_type:ident, { $($setter:tt)* }) => {
        pub struct $struct_name(pub std::sync::Arc<serenity::prelude::TypeMap>);
        impl $struct_name {
            pub fn set(shards: usize, data: &mut serenity::prelude::TypeMap) {
                let mut data_vec = Vec::with_capacity(shards);
                for _shard in 0..shards {
                    let mut type_map = serenity::prelude::TypeMap::new();
                    $($setter)*(&mut type_map);
                    data_vec.push(std::sync::Arc::new(type_map));
                }
                data.insert::<$key_type>(std::sync::Arc::new(data_vec));
            }

            pub fn initialize<F>(shards: usize, data: &mut serenity::prelude::TypeMap, callback: F)
            where
                F: FnOnce(&mut serenity::prelude::TypeMap) + Copy,
            {
                let mut data_vec = Vec::with_capacity(shards);
                for _shard in 0..shards {
                    let mut type_map = serenity::prelude::TypeMap::new();
                    callback(&mut type_map);
                    $($setter)*(&mut type_map);
                    data_vec.push(std::sync::Arc::new(type_map));
                }
                data.insert::<$key_type>(std::sync::Arc::new(data_vec));
            }

            pub async fn get(
                data: &utils::DataType,
                shard_id: serenity::all::ShardId,
            ) -> Option<std::sync::Arc<serenity::prelude::TypeMap>> {
                let data = data.read().await;
                let datas = data.get::<$key_type>().cloned()?;
                datas.get(shard_id.get() as usize).map(|p| p.clone())
            }
        }

        #[serenity::async_trait]
        impl<T> $crate::extractors::Extractor<T> for $struct_name {
            async fn extract(ctx: &serenity::all::Context, _e: &T, _p: &utils::Pointer<utils::Parser>) -> Option<Self> {
                let shard_id = ctx.shard_id;
                let pointer = $struct_name::get(&ctx.data, shard_id).await;
                pointer.map($struct_name)
            }
        }
    };
}

pub struct Commands;
impl TypeMapKey for Commands {
    type Value = CommandManager;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserGlobalType {
    Guild(GuildId, UserId),
    User(UserId),
}

#[derive(Debug, Clone)]
pub struct UserConfigHash<V>(Cache<UserGlobalType, Pointer<V>>)
where
    V: Clone + Send + Sync + 'static;

impl<V> UserConfigHash<V>
where
    V: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self(
            Cache::builder()
                .max_capacity(100_000)
                .time_to_live(Duration::from_hours(12))
                .build(),
        )
    }

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
impl<V> GlobalExtractable for UserConfigHash<V>
where
    V: Clone + Send + Sync + 'static,
{
    fn init(map: &mut TypeMap) {
        map.insert::<UserConfigHash<V>>(UserConfigHash::new());
    }

    async fn retrieve(map: &DataType) -> Option<Self> {
        map.read().await.get::<UserConfigHash<V>>().cloned()
    }
}

#[async_trait]
impl<T, V> Extractor<T> for UserConfigHash<V>
where
    V: Clone + Send + Sync + 'static,
{
    async fn extract(
        ctx: &serenity::all::Context,
        _e: &T,
        _p: &utils::Pointer<utils::Parser>,
    ) -> Option<Self> {
        GlobalExtractable::retrieve(&ctx.data).await
    }
}
