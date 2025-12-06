use framework::guilds::Messages;
use serenity::{
    all::{GuildId, Message, PartialGuild, Reaction},
    futures::channel,
};
use utils::{Pointer, debug};

use crate::{
    cache::snipe::{EditSnipes, ReactionSnipes, Snipes},
    events::message,
};

macro_rules! log_message_event {
    ($action:expr, $guild:expr, $message_id:expr) => {
        debug!(
            "{} message {} in guild {} ({})",
            $action,
            $message_id,
            $guild.name.underline(),
            $guild.id
        );
    };
}

pub async fn deleted(guild: PartialGuild, msg: Option<Message>, Snipes(snipes): Snipes) {
    if let Some(msg) = msg {
        if msg.author.bot {
            return;
        }
        let message_id = msg.id;
        let channel_id = msg.channel_id;
        if let Some(vec) = snipes.get(&channel_id).await {
            vec.write().await.push(msg);
        } else {
            snipes.insert(channel_id, Pointer::new(vec![msg])).await;
        }
        log_message_event!("Sniped", guild, message_id);
    }
}
pub async fn edited(
    guild: PartialGuild,
    update_msg: Message,
    Messages(messages): Messages,
    EditSnipes(snipes): EditSnipes,
) {
    if update_msg.author.bot {
        return;
    }
    let (channel_id, message_id) = (update_msg.channel_id, update_msg.id);
    if let Some(old_msg) = messages.get(&(channel_id, message_id)).await {
        if let Some(vec) = snipes.get(&channel_id).await {
            vec.write().await.push(old_msg);
        } else {
            snipes.insert(channel_id, Pointer::new(vec![old_msg])).await;
        }
        log_message_event!("Edit Sniped", guild, message_id);
    }
}

pub async fn reaction(
    guild: PartialGuild,
    reaction: Reaction,
    ReactionSnipes(snipes): ReactionSnipes,
) {
    let name: String = match &reaction.emoji {
        serenity::all::ReactionType::Custom { name, .. } => {
            name.clone().unwrap_or("Unknown".into())
        }
        serenity::all::ReactionType::Unicode(emoji) => emoji.clone(),
        _ => "Unknown".to_string(),
    };
    let channel_id = reaction.channel_id;
    if let Some(vec) = snipes.get(&channel_id).await {
        vec.write().await.push(reaction);
    } else {
        snipes
            .insert(channel_id, Pointer::new(vec![reaction]))
            .await;
    }
    log_message_event!("Reaction Sniped", guild, name);
}
