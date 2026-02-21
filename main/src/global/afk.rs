use chrono::DateTime;
use serde::Deserialize;
use serenity::all::{GuildId, UserId};
use utils::{deserialize_date, deserialize_id, deserialize_optional_id};

#[derive(Deserialize, Clone, Debug)]
pub struct AfkStatus {
    #[serde(deserialize_with = "deserialize_id")]
    pub user_id: UserId,
    #[serde(deserialize_with = "deserialize_optional_id")]
    pub guild_id: Option<GuildId>,
    #[serde(deserialize_with = "deserialize_date")]
    pub created_at: DateTime<chrono::Utc>,
    pub reason: String,
}
