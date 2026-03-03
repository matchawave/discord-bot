use serenity::all::{GuildId, ShardId, UnavailableGuild};
use utils::{error, info};

use crate::global::backend_http::BackendHttp;

pub async fn create(shard_id: ShardId, guild_id: GuildId, backend_http: BackendHttp) {
    let path = format!("api/guild/{}?shard_id={}", guild_id, shard_id);
    match backend_http.post::<(), ()>(&path, &()).await {
        Ok(_) => info!("Registered guild {guild_id} (Shard {shard_id})"),
        Err(e) => error!("Failed to register guild {guild_id} (Shard {shard_id})\n{e}"),
    }
}

pub async fn delete(
    unavailable_guild: UnavailableGuild,
    guild_id: GuildId,
    backend_http: BackendHttp,
) {
    if unavailable_guild.unavailable {
        let path = format!("api/guild/{}?unavailable=true", guild_id);
        match backend_http.post::<(), ()>(&path, &()).await {
            Ok(_) => info!("Marked guild {guild_id} as unavailable"),
            Err(e) => error!("Failed to mark guild {guild_id} as unavailable: {e}"),
        }
    } else {
        let path = format!("api/guild/{}", guild_id);
        match backend_http.delete::<()>(&path).await {
            Ok(_) => info!("Disabled guild {guild_id}"),
            Err(e) => error!("Failed to disable guild {guild_id}: {e}"),
        }
    }
}
