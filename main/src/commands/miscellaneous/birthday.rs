#![allow(dead_code)]

use std::str::FromStr;

use chrono::{Datelike, NaiveDate, TimeZone};
use framework::{
    command::{CommandBuilder, CommandResult, ICommand},
    extractors::InteractionOptions,
    global::GlobalCache,
};
use serde::Serialize;
use serenity::all::{
    Colour, CommandDataOptionValue, CommandOptionType, CreateCommandOption, CreateEmbed,
    FormattedTimestamp, Mentionable, Timestamp, UserId,
};
use utils::{MemberOption, ResponseError, error, suffix};

use crate::{
    global::{backend_http::BackendHttp, birthday::Birthday},
    success,
};

const NAME: &str = "birthday";
const DESCRIPTION: &str = "Set your birthday and get a surprise on your special day!";

pub fn command() -> ICommand {
    let date_option = CreateCommandOption::new(
        CommandOptionType::String,
        "date",
        "The date of the birthday (MM-DD) or (YYYY-MM-DD)",
    )
    .required(true);
    let set_options =
        CreateCommandOption::new(CommandOptionType::SubCommand, "set", "Set your birthday")
            .add_sub_option(date_option);

    CommandBuilder::default()
        .options(&[set_options])
        .slash(interaction)
        .legacy(legacy)
        .build(NAME, DESCRIPTION)
}

#[derive(Debug, Serialize)]
struct NewBirthday {
    month: u8,
    day: u8,
    year: Option<u16>,
}

async fn interaction(
    user_id: UserId,
    options: InteractionOptions,
    backend_http: BackendHttp,
) -> CommandResult<CreateEmbed> {
    // Handle interaction command
    if let Some(CommandDataOptionValue::SubCommand(set_option)) = options.get("set")
        && let Some(date_option) = set_option.first()
        && let CommandDataOptionValue::String(date_str) = &date_option.value
    {
        // Many valide options for date. (MM-DD, YYYY-MM-DD, MM/DD, YYYY/MM/DD, MM.DD, YYYY.MM.DD)
        let (month, day, year) = check_date_formats(date_str)?;
        let new_birthday = NewBirthday { month, day, year };
        let path = format!("api/birthday/{}", user_id);
        let response: Option<Birthday> =
            (backend_http.post(&path, &new_birthday).await).map_err(|e| {
                error!("Error updating birthday for user {user_id}: {e}");
                ResponseError::new_silent(format!(
                    "{}: Failed to update birthday",
                    user_id.mention()
                ))
            })?;
        if let Some(resp_bday) = response {
            let today = chrono::Utc::now();
            let birthday = match resp_bday.year {
                Some(year) => NaiveDate::from_ymd_opt(
                    year.into(),
                    resp_bday.month.into(),
                    resp_bday.day.into(),
                ),
                None => NaiveDate::from_ymd_opt(
                    today.year(),
                    resp_bday.month.into(),
                    resp_bday.day.into(),
                ),
            }
            .ok_or_else(|| "Invalid date returned from backend".to_string())?;

            // Output message format: Month Name, Day with suffix
            let output_date = birthday
                .format("%B%e{S}")
                .to_string()
                .replace("{S}", suffix(birthday.day() as u64));

            return Ok(Some(success!(
                user_id.mention(),
                "Your birthday has been set to **{}**",
                output_date
            )));
        }

        return Err(ResponseError::new_silent(
            "Failed to set birthday. Please try again later.",
        ));
    }
    Err(ResponseError::warn_silent(
        "Invalid subcommand. Use `/birthday set` to set your birthday.",
    ))
}

async fn legacy(
    user_id: UserId,
    options: Vec<String>,
    cache: GlobalCache<Birthday>,
    backend_http: BackendHttp,
) -> CommandResult<CreateEmbed> {
    if let Some(first_option) = options.first()
        && first_option.eq_ignore_ascii_case("list")
    {
        // This is to handle guild's birthday listing
        return Ok(Some(
            CreateEmbed::default().description("To view a list of birthdays in this guild"),
        ));
    }
    let target_id = match options.first() {
        Some(s) => *(MemberOption::from_str(s)?),
        None => user_id,
    };
    let is_self_command = user_id == target_id;

    // Check the cache first, if the user's birthday is not in the cache
    //fetch it from the backend and cache it for future use
    let birthday = match cache.get(None, target_id).await {
        Some(birthday) => birthday,
        None => {
            let path = format!("api/birthday/{}", target_id);
            let birthday: Option<Birthday> = (backend_http.get(&path).await).map_err(|e| {
                error!("Error fetching birthday for user {target_id}: {e}");
                ResponseError::new_silent(format!(
                    "{}: Failed to fetch birthday",
                    target_id.mention()
                ))
            })?;
            if let Some(birthday) = birthday {
                Some(cache.insert(None, target_id, birthday.clone()).await)
            } else {
                None
            }
        }
    };

    if let Some(birthday) = birthday {
        let current_date = chrono::Utc::now().date_naive();
        let embed = CreateEmbed::default().color(Colour::MAGENTA);

        let birthday = birthday.read().await;
        let mut output_string = format!("{}: ", user_id.mention());

        let age = if let Some(year) = birthday.year {
            let now = chrono::Utc::now();
            let current_year = now.year() as u16;
            Some(current_year - year)
        } else {
            None
        };
        let target_mention = target_id.mention();
        // Check if today is the user's birthday
        if birthday.month == current_date.month() as u8 && birthday.day == current_date.day() as u8
        {
            if let Some(age) = age {
                if is_self_command {
                    output_string.push_str(&format!("You are turning {} years old, ", age));
                } else {
                    output_string
                        .push_str(&format!("{target_mention} is turning {age} years old, ",));
                }
            } else if is_self_command {
                output_string.push_str("It's your birthday, ");
            } else {
                output_string.push_str(&format!("It's {target_mention}'s birthday, "));
            }
            output_string.push_str("Happy Birthday! 🎉🎂");

            return Ok(Some(embed.description(output_string)));
        }

        // Check if the birthday has already passed this year
        let mut next_birthday_year = current_date.year() as u16;
        if birthday.month < current_date.month() as u8
            || (birthday.month == current_date.month() as u8
                && birthday.day < current_date.day() as u8)
        {
            next_birthday_year += 1;
        }

        if let Some(next_birthday) = chrono::NaiveDate::from_ymd_opt(
            next_birthday_year.into(),
            birthday.month.into(),
            birthday.day.into(),
        ) {
            let next_birthday_date =
                chrono::Utc.from_utc_datetime(&next_birthday.and_hms_opt(0, 0, 0).unwrap());
            let timestamp: Timestamp =
                Timestamp::from_unix_timestamp(next_birthday_date.timestamp())
                    .map_err(|_| "Failed to create timestamp for next birthday".to_string())?;

            let formatted_timestamp = FormattedTimestamp::new(
                timestamp,
                Some(serenity::all::FormattedTimestampStyle::RelativeTime),
            );

            if let Some(age) = age {
                if is_self_command {
                    output_string.push_str("Your");
                } else {
                    output_string.push_str(&target_mention.to_string());
                }
                output_string.push_str(&format!(
                    " **{}{}** birthday is {}",
                    age + 1,
                    suffix((age + 1) as u64),
                    formatted_timestamp
                ));
            } else if is_self_command {
                output_string.push_str(&format!("Your birthday is {}", formatted_timestamp));
            } else {
                output_string.push_str(&format!(
                    "{}'s birthday is {}",
                    target_mention, formatted_timestamp
                ));
            }
        }

        return Ok(Some(embed.description(output_string)));
    }
    if is_self_command {
        return Err(ResponseError::new(
            "You do not have a birthday set.\nUse `/birthday set` to set your birthday.",
        ));
    }
    Err(ResponseError::warn(format!(
        "{} does not have a birthday set.",
        target_id.mention()
    )))
}

pub fn check_date_formats(date_str: &str) -> Result<(u8, u8, Option<u16>), ResponseError> {
    let dash_date: Vec<&str> = date_str.split("-").collect();
    parse_date_function(&dash_date)
        .or_else(|_| {
            let dot_date: Vec<&str> = date_str.split(".").collect();
            parse_date_function(&dot_date)
        })
        .or_else(|_| {
            let slash_date: Vec<&str> = date_str.split("/").collect();
            parse_date_function(&slash_date)
        })
}

fn parse_date_function(vector: &[&str]) -> Result<(u8, u8, Option<u16>), ResponseError> {
    let month: u8;
    let day: u8;
    let mut year = None;
    if vector.len() == 2 {
        // This is just MM-DD format, no year provided
        month = (vector[1].parse::<u8>()).map_err(|_| "Invalid month format")?;
        day = (vector[0].parse::<u8>()).map_err(|_| "Invalid day format")?;
    } else if vector.len() == 3 {
        // This could be either YYYY-MM-DD or MM-DD-YYYY,
        // we need to check the length of the first and last elements to determine which is which
        // Month should always be in the middle, frick the american's date format
        let first = vector[0];
        let last = vector[2];
        month = vector[1]
            .parse::<u8>()
            .map_err(|_| "Invalid month format")?;

        if first.len() <= 2 {
            day = first.parse::<u8>().map_err(|_| "Invalid day format")?;
            year = Some(last.parse::<u16>().map_err(|_| "Invalid year format")?);
        } else if first.len() <= 4 {
            day = last.parse::<u8>().map_err(|_| "Invalid day format")?;
            year = Some(first.parse::<u16>().map_err(|_| "Invalid year format")?);
        } else {
            return Err("Invalid date format".into());
        }
    } else {
        return Err("Invalid date format".into());
    }

    if day == 0 || day > 31 {
        return Err("Day must be between 1 and 31".into());
    }

    if month == 0 || month > 12 {
        return Err("Month must be between 1 and 12".into());
    }

    if let Some(year) = year {
        let now = chrono::Utc::now();
        let current_year = now.year();
        if year < 1900 || year > current_year as u16 {
            return Err(format!("Year must be between 1900 and {}", current_year).into());
        }

        let date = chrono::NaiveDate::from_ymd_opt(year.into(), month.into(), day.into())
            .ok_or_else(|| "Invalid date".to_string())?; // This will return None if the date is invalid (e.g., February 30th on a non-leap year)

        // Check if the date is in the future
        let today = now.date_naive();
        if date > today {
            return Err("You cannot be born in the future!".into());
        }
    }
    Ok((month, day, year))
}
