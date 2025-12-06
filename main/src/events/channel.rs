use framework::guilds::Channels;
use serenity::all::{GuildChannel, PartialGuild};
use utils::info;

macro_rules! log_channel_event {
    ($action:expr, $guild:expr, $channel:expr) => {
        info!(
            "{} channel {} ({}) in guild {} ({})",
            $action,
            $channel.name.underline(),
            $channel.id,
            $guild.name.underline(),
            $guild.id
        );
    };
}

pub async fn create(guild: PartialGuild, channel: GuildChannel, Channels(channels): Channels) {
    channels.write().await.insert(channel.id, channel.clone());
    log_channel_event!("Created", guild, channel);
}
pub async fn update(guild: PartialGuild, channel: GuildChannel, Channels(channels): Channels) {
    channels.write().await.insert(channel.id, channel.clone());
    log_channel_event!("Updated", guild, channel);
}

pub async fn delete(guild: PartialGuild, channel: GuildChannel, Channels(channels): Channels) {
    channels.write().await.remove(&channel.id);
    log_channel_event!("Deleted", guild, channel);
}
