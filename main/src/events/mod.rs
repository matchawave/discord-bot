mod channel;
mod guild;
mod member;
mod message;
mod snipe;
mod voice_states;

use framework::event::EventManager;
use utils::DiscordEvent;

pub fn create_event_handler(shard_count: usize) -> EventManager {
    let event_manager = EventManager::new(shard_count);
    // Register event handlers here
    event_manager.add_handler(DiscordEvent::MessageCreate, message::create);

    event_manager.add_handler(DiscordEvent::MessageDelete, snipe::deleted);
    event_manager.add_handler(DiscordEvent::MessageUpdate, snipe::edited);
    event_manager.add_handler(DiscordEvent::ReactionRemove, snipe::reaction);

    event_manager.add_handler(DiscordEvent::MessageUpdate, message::update);
    event_manager.add_handler(DiscordEvent::MessageDelete, message::delete);

    event_manager.add_handler(DiscordEvent::GuildCreate, guild::create);
    event_manager.add_handler(DiscordEvent::GuildUpdate, guild::update);
    event_manager.add_handler(DiscordEvent::GuildDelete, guild::delete);

    event_manager.add_handler(DiscordEvent::ChannelCreate, channel::create);
    event_manager.add_handler(DiscordEvent::ChannelUpdate, channel::update);
    event_manager.add_handler(DiscordEvent::ChannelDelete, channel::delete);

    event_manager.add_handler(DiscordEvent::VoiceStateUpdate, voice_states::channels);
    event_manager.add_handler(DiscordEvent::VoiceStateUpdate, voice_states::create);
    event_manager.add_handler(DiscordEvent::VoiceStateUpdate, voice_states::delete);

    event_manager.add_handler(DiscordEvent::GuildMemberAdd, member::add_member);
    event_manager.add_handler(DiscordEvent::GuildMemberRemove, member::subtract_member);
    event_manager.add_handler(DiscordEvent::GuildCreate, member::get_members);

    event_manager
}
