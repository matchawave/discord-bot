use framework::cache::Messages;
use serenity::all::{GuildId, Member, Message};
use utils::debug;

use crate::cache::snipe::{EditSnipes, Snipes};

pub async fn create(guild_id: GuildId, msg: Message, _messages: Messages, member: Member) {
    if member.user.bot {
        return;
    }
    let content = msg.content.clone();
    // println!("Current Bot: {:?}", bot);
    println!("Received message: {}", content);
}

pub async fn update(
    guild_id: GuildId,
    update_msg: Message,
    messages: Messages,
    snipes: EditSnipes,
    member: Member,
) {
    if member.user.bot {
        return;
    }
    let (channel_id, message_id) = (update_msg.channel_id, update_msg.id);
    if let Some(old_msg) = messages.get((channel_id, message_id)).await {
        snipes.insert(old_msg.make_clone().await).await;
        debug!("Stored old message id {} in edit snipes", message_id);
    }
    messages.insert((channel_id, message_id), update_msg).await;
    debug!("Updated message id {} in cache", message_id);
}

pub async fn delete(guild_id: GuildId, msg: Option<Message>, snipes: Snipes, messages: Messages) {
    if let Some(msg) = msg {
        if msg.author.bot {
            return;
        }
        messages.remove((msg.channel_id, msg.id)).await;
        debug!("Deleted message: {}", msg.content);
        snipes.insert(msg).await;
    }
}
