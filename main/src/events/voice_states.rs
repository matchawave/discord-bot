use framework::cache::VoiceStates;
use serenity::all::{GuildId, Member, VoiceState};

pub async fn update(
    guild_id: GuildId,
    member: Member,
    new_state: VoiceState,
    voice_states: VoiceStates,
) {
    let old_state = voice_states.get((guild_id, member.user.id)).await;
}
