use std::collections::HashMap;

use framework::{DataExtractable, DefaultExtract, extractors::Extractor};
use serenity::{
    all::{ChannelId, Message},
    prelude::TypeMapKey,
};
use utils::Pointer;

macro_rules! snipe_builder {
    ($name:ident, $base:ty) => {
        #[derive(Clone, Default, DataExtractable, DefaultExtract)]
        pub struct $name(Pointer<HashMap<ChannelId, Vec<$base>>>);

        impl TypeMapKey for $name {
            type Value = Pointer<HashMap<ChannelId, Vec<$base>>>;
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
    };
}

snipe_builder!(Snipes, Message);
snipe_builder!(EditSnipes, Message);
