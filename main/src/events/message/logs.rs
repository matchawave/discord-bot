use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::DateTime;
use framework::{
    data::{Ephemeral, Ephemerals},
    global::GlobalMap,
    guilds::Messages,
};
use serenity::all::{
    ChannelId, Colour, CreateEmbed, CreateMessage, GuildId, Member, Mentionable, Message,
    MessageId, PartialGuild, UserId,
};
use utils::{HttpType, debug, error};

use crate::{events::message, global::afk::AfkStatus};

macro_rules! log_message_event {
    ($action:expr, $guild:expr, $message_id:expr) => {
        debug!(
            "{} message id {} in guild {} ({})",
            $action,
            $message_id,
            $guild.name.underline(),
            $guild.id
        );
    };
}

pub async fn update(
    guild: PartialGuild,
    update_msg: Message,
    Messages(messages): Messages,
    member: Member,
) {
    if member.user.bot {
        return;
    }
    let (channel_id, message_id) = (update_msg.channel_id, update_msg.id);
    messages.insert((channel_id, message_id), update_msg).await;
    log_message_event!("Updated", guild, message_id);
}
