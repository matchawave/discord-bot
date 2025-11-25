use framework::{CacheExtract, CacheExtractable, Extractable, extractors::Extractor};
use moka::future::Cache;
use serenity::{
    all::{ChannelId, GuildId, Message, Reaction},
    prelude::TypeMapKey,
};

macro_rules! snipe_builder {
    ($($name:ident, $base:ty;)*) => {
        $(
            #[derive(Clone, CacheExtract, CacheExtractable)]
            #[cache(live="2h", capacity=100000)]
            pub struct $name(Cache<(GuildId, ChannelId), utils::Pointer<Vec<$base>>>);

            impl TypeMapKey for $name {
                type Value = Cache<(GuildId, ChannelId), utils::Pointer<Vec<$base>>>;
            }

            impl $name {
                pub async fn get(&self, key: (GuildId, ChannelId)) -> Option<utils::Pointer<Vec<$base>>> {
                    self.0.get(&key).await.map(|v| v.clone())
                }

                pub async fn get_cloned(&self, key: (GuildId, ChannelId)) -> Option<Vec<$base>> {
                    let p = self.0.get(&key).await?;
                    Some(p.make_clone().await)
                }

                pub async fn insert(&self, value: $base) {
                    if let Some(guild_id) = value.guild_id {
                        let entry = self.0.get(&(guild_id, value.channel_id)).await.unwrap_or_default();
                        entry.write().await.push(value);
                        let len = entry.read().await.len();
                        if len > 10 {
                            entry.write().await.remove(0);
                        }
                    }
                }

                pub async fn clear(&self, key: (GuildId, ChannelId)) {
                    self.0.invalidate(&key).await;
                }

                pub async fn clear_guild(&self, guild_id: GuildId) {
                    if let Err(e) = self.0.invalidate_entries_if(move |e, _i| {
                        e.0 == guild_id
                    }) {
                        utils::error!("[Snipe] : Failed to clear guild snipes: {}", e);
                        return;
                    }
                }
            }
        )*
    };
}

snipe_builder!(
    Snipes, Message;
    EditSnipes, Message;
    ReactionSnipes, Reaction;
);
