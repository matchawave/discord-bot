use chrono::DateTime;
use serde::Deserialize;
use serenity::all::UserId;
use utils::{deserialize_date, deserialize_id};

#[derive(Deserialize, Clone, Debug)]
pub struct AfkConfig {
    #[serde(deserialize_with = "deserialize_id")]
    pub user_id: UserId,
    pub per_guild: bool,
    pub default_reason: Option<String>,
    #[serde(deserialize_with = "deserialize_date")]
    pub created_at: DateTime<chrono::Utc>,
    #[serde(deserialize_with = "deserialize_date")]
    pub updated_at: DateTime<chrono::Utc>,
}
