use chrono::DateTime;
use serde::Deserialize;

pub fn deserialize_id<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
{
    let s = String::deserialize(deserializer)?;
    s.parse()
        .map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &"a valid ID"))
}

pub fn deserialize_optional_id<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => s.parse().map(Some).map_err(|_| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &"a valid ID")
        }),
        None => Ok(None),
    }
}

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<DateTime<chrono::Utc>, D::Error>
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
