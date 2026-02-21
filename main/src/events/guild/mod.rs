use framework::guilds::GuildMap;

use serenity::all::{PartialGuild, UnavailableGuild};
use utils::info;

use crate::cache::snipe::{EditSnipes, ReactionSnipes, Snipes};

pub mod loader;
pub mod update;

pub async fn create(guild: PartialGuild, guild_map: GuildMap) {
    info!("Joined guild {} ({})", guild.name, guild.id);
    let mut map_write = guild_map.write().await;
    map_write.insert::<Snipes>(Snipes::default().0);
    map_write.insert::<EditSnipes>(EditSnipes::default().0);
    map_write.insert::<ReactionSnipes>(ReactionSnipes::default().0);
}

pub async fn delete(unavailable_guild: UnavailableGuild, guild: PartialGuild) {
    if unavailable_guild.unavailable {
        info!("Guild {} ({}) got deleted", guild.name, guild.id);
    } else {
        info!("Bot got removed from guild {} ({})", guild.name, guild.id);
    }
}
