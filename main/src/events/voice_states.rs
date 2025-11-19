use framework::{
    cache::{Channels, HTTPGetter, VoiceStates},
    global::UserConfigHash,
};
use serenity::all::{
    ChannelId, ChannelType, CreateChannel, GuildId, Member, PartialGuild, VoiceState,
};
use utils::{Formatter, Http, Parser, Pointer, debug, error};

use crate::{
    configs::VoiceConfig,
    data::{voice_channels::ChannelMembers, voice_master::VoiceMasters},
};

pub async fn channels(
    guild_id: GuildId,
    member: Member,
    new_state: VoiceState,
    voice_states: VoiceStates,
    channel_members: ChannelMembers,
) {
    let old_state = voice_states.get_cloned((guild_id, member.user.id)).await;
    if let Some(old) = old_state
        && let Some(old_channel_id) = old.channel_id
    {
        channel_members
            .remove(guild_id, old_channel_id, member.user.id)
            .await;
    } else {
        channel_members.remove_user(member.user.id).await;
    }
    if let Some(new_channel_id) = new_state.channel_id {
        channel_members
            .insert(guild_id, new_channel_id, member.user.id)
            .await;
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn delete(
    http: Http,
    guild_id: GuildId,
    member: Member,
    states: VoiceStates,
    voice_masters: VoiceMasters,
    channels_repo: Channels,
    channel_members: ChannelMembers,
) {
    // When a user leaves a voice channel
    let channels = match channels_repo.get_cloned(guild_id).await {
        Some(c) => c,
        None => match channels_repo.fetch(&http, guild_id).await {
            Some(c) => c.make_clone().await,
            None => {
                error!("Failed to fetch channels for guild {}", guild_id);
                return;
            }
        },
    };
    let old = states.get_cloned((guild_id, member.user.id)).await;

    debug!(
        "Voice state delete event for user {} in guild {}",
        member.user.id, guild_id
    );

    let Some(channel_id) = old.and_then(|o| o.channel_id) else {
        return;
    };

    // let Some(old_channel)

    if let Some(old_channel) = channels.get(&channel_id)
        && old_channel.kind == ChannelType::Voice // Ensure it was a voice channel
        && let Some(voice_master) = voice_masters.get_cloned(guild_id).await // Get voice master config
        && voice_master.get_active(channel_id).is_some() // Check if it was an active channel
        && voice_master.is_master(channel_id).is_none() // Ensure it is not a master channel
        && let Some(members) = channel_members.get_cloned(guild_id, channel_id).await
        && members.is_empty() // Check if channel is now empty
        && let Some(voice_master_ptr) = voice_masters.get(&guild_id).await
    {
        channel_members.remove_channel(guild_id, channel_id).await; // Clean up channel members data
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
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    http: Http,
    guild: PartialGuild,
    member: Member,
    new: VoiceState,
    voice_masters: VoiceMasters,
    channels_repo: Channels,
    configs: UserConfigHash<VoiceConfig>,
    parser: Pointer<Parser>,
) {
    if let Some(new_channel_id) = new.channel_id
        && let Some(voice_master) = voice_masters.get_cloned(guild.id).await
        && let Some(master) = voice_master.is_master(new_channel_id)
    // Check if joined channel is a master channel
    {
        let guild_id = guild.id;
        {
            let parser = parser.clone();
            let mut parser = parser.write().await;
            parser.with_guild(guild.clone());
            parser.with_member(member.clone());
        }

        let channels = match channels_repo.get(guild_id).await {
            Some(c) => c,
            None => match channels_repo.fetch(&http, guild_id).await {
                Some(c) => c,
                None => {
                    error!("Failed to fetch channels for guild {}", guild_id);
                    return;
                }
            },
        };
        // User Joined voice channel, check if it is master channel
        let new_channel = match { channels.read().await.get(&new_channel_id).cloned() } {
            Some(c) => c,
            None => match channels_repo.fetch(&http, guild_id).await {
                Some(_) => match channels.read().await.get(&new_channel_id).cloned() {
                    Some(ch) => ch,
                    None => {
                        error!(
                            "Failed to fetch channel {} for guild {}",
                            new_channel_id, guild_id
                        );
                        return;
                    }
                },
                None => {
                    error!("Failed to fetch channels for guild {}", guild_id);
                    return;
                }
            },
        };

        if new_channel.kind != ChannelType::Voice {
            return; // Not a voice channel (technical not possible, but just in case)
        }

        let config = match voice_master.get_config(new_channel_id) {
            Some(c) => Some(c.clone()),
            None => configs.get_cloned(guild_id, member.user.id).await,
        };

        let parent_id = master.or(new_channel.parent_id);

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

                if let Some(voice_master_ptr) = voice_masters.get(&guild_id).await {
                    let mut voice_master = voice_master_ptr.write().await;
                    voice_master.insert_active(created.id, member.user.id);
                }
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
