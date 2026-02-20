use std::{collections::HashMap, time::Instant};

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{GuildId, UserId},
    model::user,
};
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
