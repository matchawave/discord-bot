use std::{collections::HashMap, sync::Arc};

use framework::{
    command::CommandAction,
    data::{Data, DataExt},
    extractors::Extractor,
};
use serenity::{
    all::{ChannelId, Event, Message},
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};
use utils::Pointer;

macro_rules! snipe_builder {
    ($name:ident, $base:ty) => {
        #[derive(Clone, Default)]
        pub struct $name(Pointer<HashMap<ChannelId, Vec<$base>>>);

        impl TypeMapKey for $name {
            type Value = Pointer<HashMap<ChannelId, Vec<$base>>>;
        }

        impl DataExt for $name {
            fn init(map: &mut TypeMap) {
                map.insert::<$name>(Pointer::new(HashMap::new()));
            }

            fn retrieve(map: &Arc<TypeMap>) -> Self {
                map.get::<$name>()
                    .cloned()
                    .map($name)
                    .expect(concat!(stringify!($name), " data not initialized"))
            }
        }

        impl $name {
            pub async fn get(&self, channel_id: ChannelId) -> Option<Vec<$base>> {
                let map = self.0.read().await;
                map.get(&channel_id).cloned()
            }

            pub async fn insert(&self, value: $base) {
                let mut map = self.0.write().await;
                let entry = map.entry(value.channel_id).or_insert_with(Vec::new);
                entry.push(value);
                if entry.len() > 10 {
                    entry.remove(0);
                }
            }
        }

        #[async_trait]
        impl Extractor<CommandAction> for $name {
            async fn extract(
                ctx: &serenity::prelude::Context,
                _ev: &CommandAction,
                _p: &Pointer<utils::Parser>,
            ) -> Option<Self> {
                let shard_id = ctx.shard_id;
                Data::get(&ctx.data, shard_id)
                    .await
                    .and_then(|d| d.get::<Self>().cloned())
                    .map(Self)
            }
        }

        #[async_trait]
        impl Extractor<Event> for $name {
            async fn extract(
                ctx: &serenity::prelude::Context,
                _ev: &Event,
                _p: &Pointer<utils::Parser>,
            ) -> Option<Self> {
                let shard_id = ctx.shard_id;
                Data::get(&ctx.data, shard_id)
                    .await
                    .and_then(|d| d.get::<Self>().cloned())
                    .map(Self)
            }
        }
    };
}

snipe_builder!(Snipes, Message);
snipe_builder!(EditSnipes, Message);
