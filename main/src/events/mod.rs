mod guild;
mod member;
mod message;
mod voice_states;

use framework::event::EventManager;
use utils::DiscordEvent;

pub fn create_event_handler(shard_count: usize) -> EventManager {
    let event_manager = EventManager::new(shard_count);

    // Register event handlers here
    event_manager.add_handler(DiscordEvent::MessageCreate, message::create);
    event_manager.add_handler(DiscordEvent::MessageUpdate, message::update);
    event_manager.add_handler(DiscordEvent::MessageDelete, message::delete);

    event_manager.add_handler(DiscordEvent::GuildCreate, guild::create);
    event_manager.add_handler(DiscordEvent::GuildUpdate, guild::update);
    event_manager.add_handler(DiscordEvent::GuildDelete, guild::delete);

    event_manager.add_handler(DiscordEvent::VoiceStateUpdate, voice_states::update);

    event_manager.add_handler(DiscordEvent::GuildMemberAdd, member::add_member);
    event_manager.add_handler(DiscordEvent::GuildMemberRemove, member::subtract_member);
    event_manager.add_handler(DiscordEvent::GuildCreate, member::get_members);
    event_manager.add_handler(DiscordEvent::GuildDelete, member::remove_members);

    event_manager
}
