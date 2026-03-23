use chrono::DateTime;
use serde::Deserialize;
use serenity::all::{ChannelId, Guild, GuildId, UserId};
use utils::{deserialize_date, deserialize_id, deserialize_optional_id};

#[derive(Deserialize, Clone, Debug)]
pub struct Reminder {
    id: u64,
    #[serde(deserialize_with = "deserialize_id")]
    user_id: UserId,
    #[serde(deserialize_with = "deserialize_id")]
    guild_id: GuildId,
    #[serde(deserialize_with = "deserialize_optional_id")]
    channel_id: Option<ChannelId>,
    message: String,
    #[serde(deserialize_with = "deserialize_date")]
    remind_at: DateTime<chrono::Utc>,
    #[serde(deserialize_with = "deserialize_date")]
    created_at: DateTime<chrono::Utc>,
}
