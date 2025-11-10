use serenity::all::{Context, CurrentUser};

use crate::extractors::CurrentBot;

pub async fn update_bot(ctx: &Context, user: &CurrentUser) {
    if !(CurrentBot::is_set(&ctx.data).await) {
        CurrentBot::set(&ctx.data, user.clone().into()).await;
    }
}
