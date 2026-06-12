use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::DateTime;
use framework::{
    command::CommandName,
    data::{Ephemeral, Ephemerals},
    event::EventResult,
    guilds::{HTTPGetter, Messages},
};
use serenity::all::{
    ChannelId, Colour, CreateEmbed, CreateMessage, FormattedTimestamp, FormattedTimestampStyle,
    GuildId, Mentionable, Message, MessageReferenceKind, PartialGuild, Timestamp, UserId,
};
use utils::{HttpType, error};

use crate::{global::backend_http::BackendHttp, processes::AfkInstance};

const AFK_MAX_MENTIONED_USERS: usize = 10; // ! Discord only allows up to 10 embeds per message 

#[allow(clippy::too_many_arguments)]
pub async fn check(
    guild_id: GuildId,
    channel_id: ChannelId,
    message: Message,
    user_id: UserId,
    afk_instance: AfkInstance,
    ephemerals: Ephemerals,
    http: HttpType,
    backend_http: BackendHttp,
) -> EventResult {
    check_function(
        guild_id,
        channel_id,
        Some(message),
        user_id,
        afk_instance,
        ephemerals,
        http,
        backend_http,
    )
    .await
}

pub async fn cmd_check(
    CommandName(cmd_name): CommandName,
    guild_id: GuildId,
    channel_id: ChannelId,
    user_id: UserId,
    afk_instance: AfkInstance,
    ephemerals: Ephemerals,
    http: HttpType,
    backend_http: BackendHttp,
) -> EventResult {
    if cmd_name == "afk" {
        return Ok(None); // Don't check AFK status on the AFK command itself
    }
    check_function(
        guild_id,
        channel_id,
        None, // No message reference for command checks
        user_id,
        afk_instance,
        ephemerals,
        http,
        backend_http,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn check_function(
    guild_id: GuildId,
    channel_id: ChannelId,
    message: Option<Message>,
    user_id: UserId,
    AfkInstance(map): AfkInstance,
    Ephemerals(ephemerals): Ephemerals,
    http: HttpType,
    backend_http: BackendHttp,
) -> EventResult {
    if let Some(afk_status) = map.write().await.remove(Some(guild_id), user_id) {
        let duration_str = calculate_duration(afk_status.created_at);

        let content = format!(
            "{}: Welcome back, you were away for **{}**",
            user_id.mention(),
            duration_str
        );

        let embed = CreateEmbed::default()
            .description(content)
            .colour(Colour::LIGHT_GREY);

        let mut msg = CreateMessage::default().add_embed(embed);
        if let Some(message) = message {
            msg = msg.reference_message(&message);
        }

        tokio::spawn(async move {
            let sent_msg = match channel_id.send_message(http, msg).await {
                Ok(sent_msg) => sent_msg,
                Err(e) => {
                    error!(
                        "Failed to send AFK return message for user {}: {}",
                        user_id, e
                    );
                    return;
                }
            };

            let k = Ephemeral::new(&sent_msg);
            (ephemerals.write().await).insert(k, Instant::now() + Duration::from_secs(5));

            let mut path = format!("api/afk/user/{}", user_id);
            if let Some(g_id) = afk_status.guild_id {
                path.push_str(&format!("?guild_id={g_id}"));
            }
            if let Err(e) = backend_http.delete::<(), ()>(&path, &()).await {
                error!("Failed to delete AFK status for user {}: {}", user_id, e);
            }
        });
    }

    Ok(None)
}

/// This function is used to calculate the duration of a user's AFK status and format it as a human-readable string. It takes the created_at timestamp of the AFK status and calculates the difference from the current time, returning a string that indicates how long the user was away. For example, it might return "Welcome back, you were away for 32 minutes and 21 seconds".
/// It only calculates outputs 2 units of time, so if the user was away for 1 hour, 32 minutes, and 21 seconds, it would return "1 hour and 32 minutes". If the user was away for less than a minute, it would return "21 seconds".
fn calculate_duration(created_at: DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(created_at);
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;

    let mut output_time = Vec::new();
    if hours > 0 {
        output_time.push(format!("{} hr{}", hours, if hours == 1 { "" } else { "s" }));
    }
    if minutes > 0 {
        output_time.push(format!(
            "{} min{}",
            minutes,
            if minutes == 1 { "" } else { "s" }
        ));
    }
    if output_time.len() < 2 && seconds > 0 {
        output_time.push(format!(
            "{} sec{}",
            seconds,
            if seconds == 1 { "" } else { "s" }
        ));
    }

    output_time.join(" ")
}

pub async fn check_mentions(
    guild: PartialGuild,
    message: Message,
    AfkInstance(map): AfkInstance,
    http: HttpType,
    messages: Messages,
) -> EventResult {
    let mut mentioned_ids = Vec::with_capacity(AFK_MAX_MENTIONED_USERS); // Pre-allocate for up to 10 mentions
    if let Some(reference) = &message.message_reference
        && reference.kind == MessageReferenceKind::Default
        && let Some(msg_id) = reference.message_id
        && let Some(searched_msg) = messages.fetch(&http, (message.channel_id, msg_id)).await
    // This is a replied message, not a crosspost
    {
        mentioned_ids.push(searched_msg.author.id);
    }

    for user in message.mentions.iter() {
        if mentioned_ids.len() >= AFK_MAX_MENTIONED_USERS {
            break; // Limit to 10 mentions
        }
        if !mentioned_ids.contains(&user.id) {
            mentioned_ids.push(user.id);
        }
    }

    let mut output_embeds = Vec::with_capacity(mentioned_ids.len());
    for user_id in mentioned_ids {
        if let Some(afk_status) = map.read().await.get(Some(guild.id), user_id) {
            let afk_status = afk_status.read().await; // Clone the AFK status to avoid holding the lock

            let formatted = FormattedTimestamp::new(
                Timestamp::from(afk_status.created_at),
                Some(FormattedTimestampStyle::RelativeTime),
            );
            let reason = &afk_status.reason;

            let content = format!("{}: is AFK: **{reason}** - {formatted}", user_id.mention());
            let embed = CreateEmbed::default()
                .description(content)
                .colour(Colour::LIGHT_GREY);

            output_embeds.push(embed);
        }
    }

    if output_embeds.is_empty() {
        return Ok(None);
    }

    let msg = (CreateMessage::default().embeds(output_embeds)).reference_message(&message);
    tokio::spawn(async move {
        if let Err(e) = message.channel_id.send_message(http, msg).await {
            error!("Failed to send AFK mention message: {e}");
        }
    });

    Ok(None)
}
