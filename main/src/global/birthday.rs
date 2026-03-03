use chrono::DateTime;
use serde::Deserialize;
use serenity::all::UserId;
use utils::{deserialize_date, deserialize_id};

#[derive(Deserialize, Clone, Debug)]
pub struct Birthday {
    #[serde(deserialize_with = "deserialize_id")]
    pub user_id: UserId,
    pub month: u8,
    pub day: u8,
    pub year: Option<u16>,

    #[serde(deserialize_with = "deserialize_date")]
    pub created_at: DateTime<chrono::Utc>,
    #[serde(deserialize_with = "deserialize_date")]
    pub updated_at: DateTime<chrono::Utc>,
}
