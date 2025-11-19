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
use utils::Http;
pub use voice_state::*;

use crate::{DataExtractable, sharded_data};

sharded_data!(Cache, Caches, { set_caches });

struct Caches;
impl TypeMapKey for Caches {
    type Value = Arc<Vec<Arc<TypeMap>>>;
}

#[macro_export]
macro_rules! cached {
    ($struct_name: ident, $cache_type:ty, $key_type:ty) => {
        use $crate::DataExtractable;

        pub struct $struct_name(moka::future::Cache<$key_type, utils::Pointer<$cache_type>>);

        impl serenity::prelude::TypeMapKey for $struct_name {
            type Value = moka::future::Cache<$key_type, utils::Pointer<$cache_type>>;
        }

        impl $struct_name {
            pub async fn insert(&self, key: $key_type, value: $cache_type) {
                self.0.insert(key, utils::Pointer::new(value)).await;
            }

            pub async fn remove(&self, key: $key_type) {
                self.0.invalidate(&key).await;
            }

            pub async fn get(&self, key: $key_type) -> Option<utils::Pointer<$cache_type>> {
                self.0.get(&key).await.map(|v| v.clone())
            }

            pub async fn get_cloned(&self, key: $key_type) -> Option<$cache_type>
            where
                $cache_type: Clone,
            {
                match self.get(key).await {
                    Some(v) => Some(v.make_clone().await),
                    None => None,
                }
            }

            pub async fn contains(&self, key: $key_type) -> bool {
                self.0.contains_key(&key)
            }

            pub async fn vec(
                &self,
            ) -> Vec<(std::sync::Arc<$key_type>, utils::Pointer<$cache_type>)> {
                self.0.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            }
        }

        impl DataExtractable for $struct_name {
            fn init(map: &mut serenity::prelude::TypeMap) {
                map.insert::<$struct_name>(moka::future::Cache::builder().build());
            }

            fn retrieve(map: &std::sync::Arc<serenity::prelude::TypeMap>) -> Option<Self> {
                map.get::<$struct_name>().cloned().map(Self)
            }
        }

        #[serenity::async_trait]
        impl<T: std::marker::Sync> $crate::extractors::Extractor<T> for $struct_name {
            async fn extract(
                ctx: &serenity::all::Context,
                ev: &T,
                p: &utils::Pointer<utils::Parser>,
            ) -> Option<Self> {
                let cache = $crate::cache::Cache::extract(ctx, ev, p).await?;
                Self::retrieve(&cache.0)
            }
        }
    };
}

#[async_trait]
pub trait HTTPGetter<Key, T> {
    async fn fetch(&self, http: &Http, key: Key) -> Option<T>;
}

pub fn set_caches(data: &mut TypeMap) {
    Channels::init(data);
    Messages::init(data);
    Members::init(data);
    VoiceStates::init(data);
}
