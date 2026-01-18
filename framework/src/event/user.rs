use serenity::all::{Context, CurrentUser};

use crate::ShardData;

pub async fn update_bot(ctx: &Context, user: &CurrentUser) {
    if let Some(shard_data) = ShardData::get(ctx.shard_id, &ctx.data).await {
        shard_data.bot.set(Some(user.clone().into())).await;
    }
}
