use serenity::all::{Context, Ready};
use utils::info;

use crate::extractors::CurrentBot;

pub async fn handle(ctx: &Context, ready: &Ready) {
    if let Some(shard) = ready.shard {
        let current_shard = shard.id.get() + 1;
        info!("Shard {}/{} is connected", current_shard, shard.total);
    }
}
