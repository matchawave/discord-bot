use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::DateTime;
use framework::{
    data::{Ephemeral, Ephemerals},
    event::EventResult,
    global::GlobalMap,
    guilds::{HTTPGetter, Messages},
};
use serenity::all::{
    ChannelId, Colour, CreateEmbed, CreateMessage, FormattedTimestamp, FormattedTimestampStyle,
    GuildId, Member, Mentionable, Message, MessageId, MessageReferenceKind, PartialGuild,
    Timestamp, UserId,
};
use utils::{HttpType, ResponseError, debug, error};

use crate::{
    events::message,
    global::{
        afk::AfkStatus,
        backend_http::{self, BackendHttp},
    },
};

const AFK_MAX_MENTIONED_USERS: usize = 10;

#[allow(clippy::too_many_arguments)]
pub async fn check(
    guild_id: GuildId,
    channel_id: ChannelId,
    message: Message,
    user_id: UserId,
    map: GlobalMap<AfkStatus>,
    ephemerals: Arc<Ephemerals>,
    http: HttpType,
    backend_http: BackendHttp,
) -> EventResult {
    if let Some(afk_status) = map.remove(guild_id, user_id).await {
        let duration_str = calculate_duration(afk_status.created_at);

        let content = format!(
            "{}: Welcome back, you were away for **{}**",
            user_id.mention(),
            duration_str
        );

        let embed = CreateEmbed::default()
            .description(content)
            .colour(Colour::LIGHT_GREY);

        let msg = CreateMessage::default()
            .add_embed(embed)
            .reference_message(&message);

        let sent_msg = match channel_id.send_message(http, msg).await {
            Ok(sent_msg) => sent_msg,
            Err(e) => {
                return Err(ResponseError::Err(format!(
                    "Failed to send AFK return message: {e}"
                )));
            }
        };

        let k = Ephemeral::new(&sent_msg);
        (ephemerals.0.write().await).insert(k, Instant::now() + Duration::from_secs(5));

        if let Err(e) = backend_http
            .remove_user_afk(user_id, afk_status.guild_id)
            .await
        {
            return Err(ResponseError::Err(e));
        }
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
    map: GlobalMap<AfkStatus>,
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
        if let Some(afk_status) = map.get(guild.id, user_id).await {
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
    if let Err(e) = message.channel_id.send_message(http, msg).await {
        error!("Failed to send AFK mention message: {e}");
    }

    Ok(None)
}

// {user}: is AFK: **{reason}** - {discord_formatted_duration}
