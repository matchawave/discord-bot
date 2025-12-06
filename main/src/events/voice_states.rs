use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use framework::{
    data::{Cooldown, Cooldowns},
    extractors::CurrentBot,
    global::UserConfigHash,
    guilds::{ChannelMembers, Channels, HTTPGetter, VoiceStates},
};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator};
use serenity::all::{
    ChannelId, ChannelType, CreateChannel, GuildId, Member, PartialGuild, VoiceState,
};
use utils::{Formatter, Http, Parser, Pointer, debug, error};

use crate::{configs::VoiceConfig, data::voice_master::VoiceMaster};

pub async fn channels(
    CurrentBot(bot): CurrentBot,
    guild_id: GuildId,
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

        if let Some(i) = index_to_remove {
            if let Some(v) = channel_members.read().await.get(&old_channel_id) {
                v.write().await.remove(i);
            }
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

#[allow(clippy::too_many_arguments)]
pub async fn delete(
    http: Http,
    guild_id: GuildId,
    member: Member,
    VoiceStates(states): VoiceStates,
    VoiceMaster(voice_master_ptr): VoiceMaster,
    Channels(channels): Channels,
    ChannelMembers(channel_members): ChannelMembers,
) {
    // When a user leaves a voice channel
    let user_id = member.user.id;
    let old = states.get(&user_id).await;

    let Some(channel_id) = old.and_then(|o| o.channel_id) else {
        debug!(
            "No old channel for user {} in guild {}",
            member.user.id, guild_id
        );
        return;
    };

    let Some(old_channel) = channels.read().await.get(&channel_id).cloned() else {
        error!(
            "Old channel {} not found in cache for guild {}",
            channel_id, guild_id
        );
        return;
    };

    if old_channel.kind != ChannelType::Voice {
        // Not a voice channel (technical not possible, but just in case)
        return;
    }

    let voice_master = voice_master_ptr.make_clone().await;

    if voice_master.is_master(channel_id).is_some() {
        // It is a master channel, do not delete
        debug!(
            "Channel {} is a master channel in guild {}, not deleting",
            channel_id, guild_id
        );
        return;
    }

    if voice_master.get_active(channel_id).is_none() {
        // Not an active channel, do not delete
        debug!(
            "Channel {} is not an active voice master channel in guild {}, not deleting",
            channel_id, guild_id
        );
        return;
    }
    let mut should_delete = false;
    if let Some(members) = channel_members.read().await.get(&channel_id) {
        let members_read = members.read().await;
        if !members_read.is_empty() {
            // Channel is not empty, do not delete
            debug!(
                "Channel {} in guild {} is not empty, not deleting",
                channel_id, guild_id
            );
            return;
        }
        should_delete = true;
    }

    if should_delete {
        channel_members.write().await.remove(&channel_id); // Clean up channel members data
    }

    {
        let mut voice_master = voice_master_ptr.write().await;
        voice_master.remove_active(channel_id); // Remove from active channels
    }
    if let Err(e) = http
        .delete_channel(channel_id, Some("Voice master cleanup"))
        .await
    {
        // Delete the voice channel
        error!(
            "Failed to delete voice master channel {}: {}",
            channel_id, e
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    http: Http,
    guild: PartialGuild,
    member: Member,
    new: VoiceState,
    VoiceMaster(voice_master_ptr): VoiceMaster,
    channels: Channels,
    configs: UserConfigHash<VoiceConfig>,
    cooldowns: Arc<Cooldowns>,
    parser: Pointer<Parser>,
) {
    let v_m = voice_master_ptr.make_clone().await;
    if let Some(new_channel_id) = new.channel_id
        && let Some(master) = v_m.is_master(new_channel_id)
    // Check if joined channel is a master channel
    {
        let guild_id = guild.id;

        if let Some(cooldown_num) = master.1 {
            // Has a voice chat creating cooldown
            let cooldown = Cooldown::VoiceMaster(guild_id, new_channel_id, member.user.id);
            if (cooldowns.0.read().await).get(&cooldown).is_some() {
                debug!(
                    "User {} in guild {} is on cooldown for creating voice master channels from master channel {}",
                    member.user.id, guild_id, new_channel_id
                );
                return;
            } else {
                (cooldowns.0.write().await).insert(
                    cooldown,
                    Instant::now() + Duration::from_millis(cooldown_num),
                );
            }
        }

        {
            let parser = parser.clone();
            let mut parser = parser.write().await;
            parser.with_guild(guild.clone());
            parser.with_member(member.clone());
        }

        // User Joined voice channel, check if it is master channel
        let new_channel = match channels.0.read().await.get(&new_channel_id).cloned() {
            Some(c) => c,
            None => match channels.fetch(&http, (guild_id, new_channel_id)).await {
                Some(c) => c,
                None => {
                    error!("Failed to fetch channels for guild {}", guild_id);
                    return;
                }
            },
        };

        if new_channel.kind != ChannelType::Voice {
            return; // Not a voice channel (technical not possible, but just in case)
        }

        let config = match v_m.get_config(new_channel_id) {
            Some(c) => Some(c.clone()),
            None => configs.get_cloned(guild_id, member.user.id).await,
        };

        let parent_id = master.0.or(new_channel.parent_id);

        let channel = create_channel(
            member.user.name.clone(),
            guild_id,
            parent_id,
            config,
            parser.make_clone().await,
        );

        match http
            .create_channel(guild_id, &channel, Some("Voice master created channel"))
            .await
        {
            Ok(created) => {
                debug!(
                    "Created voice master channel {} for user {} in guild {}",
                    created.id, member.user.id, guild_id
                );
                (voice_master_ptr.write().await).insert_active(created.id, member.user.id);

                // Move user to new channel
                if let Err(e) = guild_id.move_member(http, member.user.id, created.id).await {
                    error!(
                        "Failed to move user {} to voice master channel {}: {}",
                        member.user.id, created.id, e
                    );
                }
            }
            Err(e) => {
                error!(
                    "Failed to create voice master channel for user {} in guild {} ({}): {}",
                    member.user.id, guild.name, guild_id, e
                );
            }
        }
    }
}

fn create_channel<'a>(
    user_name: String,
    guild_id: GuildId,
    parent_id: Option<ChannelId>,
    config: Option<VoiceConfig>,
    parser: Parser,
) -> CreateChannel<'a> {
    let title = match &config {
        Some(cfg) => match &cfg.name {
            Some(n) => n.format(&parser),
            None => format!("{}'s channel", user_name),
        },
        None => format!("{}'s channel", user_name),
    };
    let mut channel = CreateChannel::new(title).kind(ChannelType::Voice);
    if let Some(parent) = parent_id {
        channel = channel.category(parent);
    }
    if let Some(cfg) = &config {
        if let Some(perms) = cfg.permissions(guild_id) {
            channel = channel.permissions(perms);
        }
        if let Some(bitrate) = cfg.bitrate {
            channel = channel.bitrate(bitrate);
        }
        if let Some(user_limit) = cfg.user_limit {
            channel = channel.user_limit(user_limit);
        }
    }
    channel
}
