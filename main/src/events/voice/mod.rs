pub mod voice_master;

use framework::{
    extractors::Bot,
    guilds::{ChannelMembers, VoiceStates},
};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator};
use serenity::all::{GuildId, Member, VoiceState};
use utils::Pointer;

pub async fn channels(
    Bot(bot): Bot,
    _guild_id: GuildId,
    member: Member,
    new_state: VoiceState,
    VoiceStates(voice_states): VoiceStates,
    ChannelMembers(channel_members): ChannelMembers,
) {
    let user_id = member.user.id;
    if let Some(old) = voice_states.get(&member.user.id).await
        && let Some(old_channel_id) = old.channel_id
    {
        if let Some(new_channel_id) = new_state.channel_id
            && new_channel_id == old_channel_id
        {
            return;
        }

        if new_state.user_id == bot.id {
            // Don't track bot itself
            return;
        }

        let mut index_to_remove = None;
        if let Some(v) = channel_members.read().await.get(&old_channel_id) {
            let user_vec_read_guard = v.read().await;
            if let Some(i) = user_vec_read_guard
                .par_iter()
                .position_first(|u| user_id == *u)
            {
                index_to_remove = Some(i);
            }
        } // read locks on channel_members and v are dropped here

        if let Some(i) = index_to_remove
            && let Some(v) = channel_members.read().await.get(&old_channel_id)
        {
            v.write().await.remove(i);
        }
    } else {
        // ! TODO: Handle case where voice state was not cached
    }

    if let Some(new_channel_id) = new_state.channel_id {
        if new_state.user_id == bot.id {
            // Don't track bot itself
            return;
        }

        if let Some(v) = channel_members.read().await.get(&new_channel_id).cloned() {
            let mut user_vec_write_guard = v.write().await;
            if !user_vec_write_guard.contains(&user_id) {
                user_vec_write_guard.push(user_id);
            }
        } else {
            channel_members
                .write()
                .await
                .insert(new_channel_id, Pointer::new(vec![user_id]));
        }
    }
}
