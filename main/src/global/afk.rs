use std::{collections::HashMap, time::Instant};

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{GuildId, UserId},
    model::user,
};

pub struct AfkConfig {}

#[derive(Deserialize, Clone)]
pub struct AfkStatus {
    #[serde(deserialize_with = "deserialize_user_id")]
    pub user_id: UserId,
    #[serde(deserialize_with = "deserialize_guild_id")]
    pub guild_id: Option<GuildId>,
    #[serde(deserialize_with = "deserialize_created_at")]
    pub created_at: DateTime<chrono::Utc>,
    pub reason: String,
}

fn deserialize_user_id<'de, D>(deserializer: D) -> Result<UserId, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(|_| {
        serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &"a valid user ID")
    })
}

fn deserialize_guild_id<'de, D>(deserializer: D) -> Result<Option<GuildId>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        s.parse().map(Some).map_err(|_| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &"a valid guild ID or empty string",
            )
        })
    }
}

fn deserialize_created_at<'de, D>(deserializer: D) -> Result<DateTime<chrono::Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?; // String is in UTC RFC3339 format
    let date: DateTime<chrono::Utc> = match DateTime::parse_from_rfc3339(&s) {
        Ok(dt) => dt.into(),
        Err(_) => {
            return Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &"a valid RFC3339 datetime string",
            ));
        }
    };
    Ok(date)
}
