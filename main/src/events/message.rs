use framework::guilds::Messages;
use serenity::all::{GuildId, Member, Message, PartialGuild};
use utils::debug;

pub async fn create(guild: PartialGuild, msg: Message, _messages: Messages, member: Member) {
    if member.user.bot {
        return;
    }
    let content = msg.content.clone();
}

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

pub async fn delete(guild: PartialGuild, msg: Option<Message>, Messages(messages): Messages) {
    if let Some(msg) = msg {
        messages.remove(&(msg.channel_id, msg.id)).await;
        log_message_event!("Deleted", guild, msg.id);
    }
}
