use serenity::all::{Context, VoiceState};

use crate::{
    Extractable,
    cache::{Cache, VoiceStates},
};

pub async fn update(ctx: &Context, new: &VoiceState) {
    let shard_id = ctx.shard_id;
    if let Some(guild_id) = new.guild_id
        && let Some(cache) = Cache::get(&ctx.data, shard_id).await
        && let Some(states) = VoiceStates::retrieve(&cache)
    {
        states.insert((guild_id, new.user_id), new.clone()).await;
    }
}
