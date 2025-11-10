mod channel;
mod message;
mod user;
mod voice_state;
use std::sync::Arc;

pub use channel::*;
pub use message::*;
use serenity::{
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};
pub use user::*;
pub use voice_state::*;

use crate::sharded_data;

sharded_data!(Cache, Caches, { set_caches });

struct Caches;
impl TypeMapKey for Caches {
    type Value = Arc<Vec<Arc<TypeMap>>>;
}

#[macro_export]
macro_rules! cached {
    ($struct_name: ident, $cache_type:ty, $key_type:ty) => {
        pub struct $struct_name(utils::Pointer<moka::future::Cache<$key_type, $cache_type>>);

        impl serenity::prelude::TypeMapKey for $struct_name {
            type Value = utils::Pointer<moka::future::Cache<$key_type, $cache_type>>;
        }

        impl $struct_name {
            pub fn init(cache: &mut serenity::prelude::TypeMap) {
                cache.insert::<$struct_name>(utils::Pointer::new(
                    moka::future::Cache::builder()
                        .max_capacity(100_000)
                        .time_to_live(std::time::Duration::from_secs(60 * 60 * 12))
                        .build(),
                ));
            }

            pub async fn retrieve(cache: &std::sync::Arc<serenity::prelude::TypeMap>) -> Self {
                cache
                    .get::<$struct_name>()
                    .cloned()
                    .map($struct_name)
                    .expect(concat!(stringify!($struct_name), " cache not initialized"))
            }

            pub async fn insert(&self, key: $key_type, value: $cache_type) {
                self.0.write().await.insert(key, value).await;
            }

            pub async fn remove(&self, key: &$key_type) {
                self.0.write().await.invalidate(key).await;
            }

            pub async fn get(&self, key: &$key_type) -> Option<$cache_type> {
                self.0.read().await.get(key).await.map(|v| v.clone())
            }

            pub async fn vec(&self) -> Vec<(std::sync::Arc<$key_type>, $cache_type)> {
                self.0.read().await.iter().collect()
            }
        }

        #[serenity::async_trait]
        impl $crate::extractors::Extractor<serenity::all::Event> for $struct_name {
            async fn extract(
                ctx: &serenity::all::Context,
                ev: &serenity::all::Event,
                p: &utils::Pointer<utils::Parser>,
            ) -> Option<Self> {
                let cache = $crate::cache::Cache::extract(ctx, ev, p).await?;
                let data = cache.0.clone();
                let cached = data.get::<$struct_name>()?.clone();
                Some($struct_name(cached))
            }
        }

        #[serenity::async_trait]
        impl $crate::extractors::Extractor<$crate::command::CommandAction> for $struct_name {
            async fn extract(
                ctx: &serenity::all::Context,
                action: &$crate::command::CommandAction,
                p: &utils::Pointer<utils::Parser>,
            ) -> Option<Self> {
                let cache = $crate::cache::Cache::extract(ctx, action, p).await?;
                let data = cache.0.clone();
                let cached = data.get::<$struct_name>()?.clone();
                Some($struct_name(cached))
            }
        }
    };
}

#[async_trait]
pub trait HTTPGetter<Key, T> {
    async fn fetch(&self, http: &Arc<serenity::http::Http>, key: Key) -> Option<T>;
}

pub fn set_caches(data: &mut serenity::prelude::TypeMap) {
    Channels::init(data);
    Messages::init(data);
    Members::init(data);
    VoiceStates::init(data);
}
