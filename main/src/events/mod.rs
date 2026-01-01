mod channel;
mod guild;
mod member;
mod message;
mod snipe;
mod voice_states;

use framework::event::{EventManager, EventManagerBuilder};
use utils::DiscordEvent;

pub fn create_event_handler(shard_count: usize) -> EventManager {
    EventManagerBuilder::default()
        // Message Events
        .add_handler(DiscordEvent::MessageCreate, message::create)
        .add_handler(DiscordEvent::MessageDelete, snipe::deleted)
        .add_handler(DiscordEvent::MessageUpdate, snipe::edited)
        .add_handler(DiscordEvent::ReactionRemove, snipe::reaction)
        .add_handler(DiscordEvent::MessageUpdate, message::update)
        .add_handler(DiscordEvent::MessageDelete, message::delete)
        // Guild & Channel Events
        .add_handler(DiscordEvent::GuildCreate, guild::create)
        .add_handler(DiscordEvent::GuildUpdate, guild::update)
        .add_handler(DiscordEvent::GuildDelete, guild::delete)
        .add_handler(DiscordEvent::ChannelCreate, channel::create)
        .add_handler(DiscordEvent::ChannelUpdate, channel::update)
        .add_handler(DiscordEvent::ChannelDelete, channel::delete)
        // Voice State Events
        .add_handler(DiscordEvent::VoiceStateUpdate, voice_states::channels)
        .add_handler(DiscordEvent::VoiceStateUpdate, voice_states::create)
        .add_handler(DiscordEvent::VoiceStateUpdate, voice_states::delete)
        // Member Events
        .add_handler(DiscordEvent::GuildMemberAdd, member::add_member)
        .add_handler(DiscordEvent::GuildMemberRemove, member::subtract_member)
        .add_handler(DiscordEvent::GuildCreate, member::get_members)
        // Build and return
        .build(shard_count)
}
