use framework::guilds::Messages;
use serenity::all::{Member, Message, PartialGuild};

pub mod afk;
pub mod logs;
pub mod snipe;
pub async fn create(guild: PartialGuild, msg: Message, _messages: Messages, member: Member) {
    if member.user.bot {
        return;
    }
    let content = msg.content.clone();
}

pub async fn delete(guild: PartialGuild, msg: Option<Message>, Messages(messages): Messages) {
    if let Some(msg) = msg {
        messages.remove(&(msg.channel_id, msg.id)).await;
        // log_message_event!("Deleted", guild, msg.id);
    }
}
