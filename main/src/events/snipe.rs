use framework::cache::{HTTPGetter, Messages};
use serenity::all::{GuildId, Member, Message, Reaction};
use utils::{Http, debug};

use crate::cache::snipe::{EditSnipes, ReactionSnipes, Snipes};

pub async fn deleted(_guild_id: GuildId, msg: Option<Message>, snipes: Snipes) {
    if let Some(msg) = msg {
        if msg.author.bot || msg.guild_id.is_none() {
            return;
        }
        snipes.insert(msg).await;
    }
}
pub async fn edited(
    _guild_id: GuildId,
    update_msg: Message,
    messages: Messages,
    snipes: EditSnipes,
) {
    if update_msg.author.bot {
        return;
    }
    let (channel_id, message_id) = (update_msg.channel_id, update_msg.id);
    if let Some(old_msg) = messages.get((channel_id, message_id)).await {
        snipes.insert(old_msg.make_clone().await).await;
        debug!("Stored old message id {} in edit snipes", message_id);
    }
}
pub async fn reaction(_guild_id: GuildId, reaction: Reaction, snipes: ReactionSnipes) {
    snipes.insert(reaction).await;
}
