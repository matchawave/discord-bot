use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serenity::{all::UserId, model::user};

pub struct AfkConfig {}

#[derive(Serialize, Deserialize, Default)]
pub struct AfkStatus {
    #[serde(deserialize_with = "deserialize_user_id")]
    pub user_id: UserId,
    pub afk_since: std::time::Instant, // The time when the user went AFK
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

impl<'a> Deserialize<'a> for AfkStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let mut map = HashMap::<String, String>::deserialize(deserializer)?;
        let user_id: String = map
            .remove("user_id")
            .ok_or_else(|| serde::de::Error::missing_field("user_id"))?;
        let afk_since: String = map
            .remove("afk_since")
            .ok_or_else(|| serde::de::Error::missing_field("afk_since"))?;

        let afk_since = afk_since.parse().map_err(|_| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&afk_since),
                &"a valid timestamp",
            )
        })?;
        let user_id = user_id.parse().map_err(|_| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&user_id),
                &"a valid user ID",
            )
        })?;

        Ok(AfkStatus { user_id, afk_since })
    }
}
