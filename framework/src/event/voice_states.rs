use colored::Colorize;
use serenity::all::{Context, VoiceState};
use utils::error;

use crate::{ShardData, guilds::VoiceStates};

pub async fn update(ctx: &Context, new: &VoiceState) {
    let shard_id = ctx.shard_id;
    let shard_text = format!("(Shard {})", shard_id.get()).bold().purple();
    let seperator = "|".bold().white();

    let Some(guild_id) = new.guild_id else {
        error!(
            "{} {} Voice state update received with no guild id for user {}",
            shard_text, seperator, new.user_id
        );
        return;
    };

    let Some(shard_data) = ShardData::get(ctx).await else {
        error!(
            "{} {} Could not get shard data for voice state update in guild {} for user {}",
            shard_text, seperator, guild_id, new.user_id
        );
        return;
    };

    let Some(map) = shard_data.guilds.map(guild_id).await else {
        error!(
            "{} {} Could not get guild map for voice state update in guild {} for user {}",
            shard_text, seperator, guild_id, new.user_id
        );
        return;
    };

    let map_read = map.read().await;

    let Some(states) = map_read.get::<VoiceStates>() else {
        error!(
            "{} {} Could not get voice states cache for guild {} for user {}",
            shard_text, seperator, guild_id, new.user_id
        );
        return;
    };

    states.insert(new.user_id, new.clone()).await;
}
