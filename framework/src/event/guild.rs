use serenity::all::{Context, Guild, UnavailableGuild};
use utils::info;

use crate::ShardData;

pub async fn create(ctx: &Context, guild: &Guild) {
    if let Some(shard_data) = ShardData::get(ctx.shard_id, &ctx.data).await {
        shard_data.guilds.new_guild(guild.clone()).await;
    }
}

pub async fn delete(ctx: &Context, guild: &UnavailableGuild) {
    if let Some(shard_data) = ShardData::get(ctx.shard_id, &ctx.data).await
        && let Some(guild) = shard_data.guilds.remove_guild(guild.id).await
    {
        info!("Removed guild data for guild {} ({})", guild.name, guild.id);
    }
}
