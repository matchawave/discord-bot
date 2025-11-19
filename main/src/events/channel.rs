use framework::cache::Channels;
use serenity::all::{GuildChannel, GuildId};
use utils::{Http, error};

pub async fn create(http: Http, guild_id: GuildId, channel: GuildChannel, channels: Channels) {
    if let Some(channels) = channels.get(guild_id).await {
        channels.write().await.insert(channel.id, channel.clone());
    } else {
        match guild_id.channels(&http).await {
            Ok(map) => channels.insert(guild_id, map).await,
            Err(err) => error!("Failed to fetch channels for guild {}: {}", guild_id, err),
        }
    }
}
pub async fn update(http: Http, guild_id: GuildId, channel: GuildChannel, channels: Channels) {
    if let Some(channels) = channels.get(guild_id).await {
        channels.write().await.insert(channel.id, channel.clone());
    } else {
        match guild_id.channels(&http).await {
            Ok(map) => channels.insert(guild_id, map).await,
            Err(err) => error!("Failed to fetch channels for guild {}: {}", guild_id, err),
        }
    }
}
pub async fn delete(http: Http, guild_id: GuildId, channel: GuildChannel, channels: Channels) {
    if let Some(channels) = channels.get(guild_id).await {
        channels.write().await.remove(&channel.id);
    } else {
        match guild_id.channels(&http).await {
            Ok(map) => channels.insert(guild_id, map).await,
            Err(err) => error!("Failed to fetch channels for guild {}: {}", guild_id, err),
        }
    }
}
