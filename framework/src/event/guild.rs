use serenity::all::{Context, Guild, PartialGuild, UnavailableGuild};
use utils::info;

use crate::{
    cache::{Cache, Channels, Members, VoiceStates},
    data::{Data, DataExt, guild::Guilds},
};

use rayon::prelude::*;
macro_rules! cache_items {
    ($cache:expr, $guild:expr, $cache_struct:ident, $items:expr, $id_field:ident) => {
        let cache_instance = $cache_struct::retrieve(&$cache).await;
        info!(
            "Caching {} items into {} cache for guild {}",
            $items.len(),
            stringify!($cache_struct),
            $guild.id
        );
        for (id, item) in $items {
            let key = ($guild.id, id);
            let value = item.clone();
            cache_instance.insert(key, value).await;
        }
    };
}

macro_rules! remove_cached_items {
    ($cache:expr, $guild_id:expr, $cache_struct:ident) => {
        let cache_instance = $cache_struct::retrieve(&$cache).await;
        let keys: Vec<_> = cache_instance
            .vec()
            .await
            .into_par_iter()
            .filter_map(|(key, _)| if key.0 == $guild_id { Some(*key) } else { None })
            .collect();
        info!(
            "Removing {} items from {} cache for guild {}",
            keys.len(),
            stringify!($cache_struct),
            $guild_id
        );
        for key in keys {
            cache_instance.remove(&key).await;
        }
    };
}

pub async fn create(ctx: &Context, guild: &Guild) {
    let shard_id = ctx.shard_id;

    let (Some(data), Some(cache)) = (
        Data::get(&ctx.data, shard_id).await,
        Cache::get(&ctx.data, shard_id).await,
    ) else {
        return;
    };
    {
        let guilds = Guilds::retrieve(&data);
        let partial_guild = PartialGuild::from(guild.clone());
        guilds.insert(partial_guild).await;
    }
    let channels_cache = Channels::retrieve(&cache).await;
    channels_cache
        .insert(guild.id, guild.channels.clone())
        .await;

    #[rustfmt::skip]
    cache_items!(cache, guild, VoiceStates, guild.voice_states.clone(), user_id);
    cache_items!(cache, guild, Members, guild.members.clone(), user_id);
    // cache_items!(cache, guild, Channels, guild.channels.clone(), channel_id);
}

pub async fn delete(ctx: &Context, guild: &UnavailableGuild) {
    let shard_id = ctx.shard_id;

    let (Some(data), Some(cache)) = (
        Data::get(&ctx.data, shard_id).await,
        Cache::get(&ctx.data, shard_id).await,
    ) else {
        return;
    };
    let guilds = Guilds::retrieve(&data);
    if let Some(cached_guild) = guilds.get(&guild.id).await {
        let cached_ = cached_guild.read().await;
        info!(
            "Removing guild {} ({}) from cache",
            cached_.name, cached_.id
        );
        guilds.remove(&guild.id).await;
    };
    let channels_cache = Channels::retrieve(&cache).await;
    channels_cache.remove(&guild.id).await;

    remove_cached_items!(cache, guild.id, VoiceStates);
    remove_cached_items!(cache, guild.id, Members);
}
