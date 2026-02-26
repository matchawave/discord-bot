mod channel;
mod guild;
mod member;
mod message;
mod ready;
mod voice_states;

use framework::event::{EventManager, EventManagerBuilder};
use utils::DiscordEvent;

pub fn create_event_handler(shard_count: usize) -> EventManager {
    EventManagerBuilder::default()
        // Message Events
        .add_event(DiscordEvent::MessageCreate, message::create)
        .add_event(DiscordEvent::MessageDelete, message::snipe::deleted)
        .add_event(DiscordEvent::MessageUpdate, message::snipe::edited)
        .add_event(DiscordEvent::ReactionRemove, message::snipe::reaction)
        .add_event(DiscordEvent::MessageDelete, message::delete)
        // Guild & Channel Events
        .add_event(DiscordEvent::GuildCreate, guild::create)
        .add_event(DiscordEvent::GuildCreate, guild::loader::create)
        .add_event(DiscordEvent::GuildUpdate, guild::update::update)
        .add_event(DiscordEvent::GuildDelete, guild::delete)
        .add_event(DiscordEvent::GuildDelete, guild::loader::delete)
        .add_event(DiscordEvent::ChannelCreate, channel::create)
        .add_event(DiscordEvent::ChannelUpdate, channel::update)
        .add_event(DiscordEvent::ChannelDelete, channel::delete)
        // Voice State Events
        .add_event(DiscordEvent::VoiceStateUpdate, voice_states::channels)
        .add_event(DiscordEvent::VoiceStateUpdate, voice_states::create)
        .add_event(DiscordEvent::VoiceStateUpdate, voice_states::delete)
        // Member Events
        .add_event(DiscordEvent::GuildMemberAdd, member::add_member)
        .add_event(DiscordEvent::GuildMemberRemove, member::subtract_member)
        .add_event(DiscordEvent::GuildCreate, member::get_members)
        // Afk Events
        .add_event(DiscordEvent::MessageCreate, message::afk::check)
        .add_event(DiscordEvent::MessageCreate, message::afk::check_mentions)
        // Logs
        .add_event(DiscordEvent::MessageUpdate, message::logs::update)
        // Build and return
        .build(shard_count)
}
