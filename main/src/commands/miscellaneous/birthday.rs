#![allow(dead_code)]

use chrono::Datelike;
use framework::{
    command::{CommandBuilder, CommandResult, ICommand},
    extractors::InteractionOptions,
};
use serenity::all::{CommandDataOptionValue, CommandOptionType, CreateCommandOption};
use utils::ResponseError;

const NAME: &str = "birthday";
const DESCRIPTION: &str = "Set your birthday and get a surprise on your special day!";

const DATE_FORMATS: [&str; 8] = [
    "%m-%d", "%Y-%m-%d", "%m/%d", "%Y/%m/%d", "%m.%d", "%Y.%m.%d", "%Y%m%d", "%m%d",
];

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
        .options(vec![set_options])
        .slash(interaction)
        .legacy(legacy)
        .build(NAME, DESCRIPTION)
}

async fn interaction(options: InteractionOptions) -> CommandResult<String> {
    // Handle interaction command
    if let Some(CommandDataOptionValue::SubCommand(set_option)) = options.get("set")
        && let Some(date_option) = set_option.first()
        && let CommandDataOptionValue::String(date_str) = &date_option.value
    {
        // Many valide options for date. (MM-DD, YYYY-MM-DD, MM/DD, YYYY/MM/DD, MM.DD, YYYY.MM.DD)
        let (month, day, year) = check_date_formats(date_str)?;
        // Here you would save the birthday to your database or perform any necessary actions
        let mut output_message = format!("Your birthday has been set to: {}-{}", month, day);
        if let Some(year) = year {
            output_message.push_str(&format!("-{}", year));
        }
        return Ok(Some(output_message));
    }
    Ok(Some(
        "Hello! This is a placeholder response for the birthday command.".to_string(),
    ))
}

async fn legacy() {}

pub fn check_date_formats(date_str: &str) -> Result<(u32, u32, Option<i32>), ResponseError> {
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

fn parse_date_function(vector: &[&str]) -> Result<(u32, u32, Option<i32>), ResponseError> {
    let month: u32;
    let day: u32;
    let mut year = None;
    if vector.len() == 2 {
        month = (vector[0].parse::<u32>()).map_err(|_| "Invalid month format")?;
        day = (vector[1].parse::<u32>()).map_err(|_| "Invalid day format")?;
    } else if vector.len() == 3 {
        let first = vector[0];
        let last = vector[2];
        month = vector[1]
            .parse::<u32>()
            .map_err(|_| "Invalid month format")?;

        if first.len() == 4 {
            year = Some(first.parse::<i32>().map_err(|_| "Invalid year format")?);
            day = last.parse::<u32>().map_err(|_| "Invalid day format")?;
        } else if first.len() == 2 {
            day = first.parse::<u32>().map_err(|_| "Invalid day format")?;
            year = Some(last.parse::<i32>().map_err(|_| "Invalid year format")?);
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
        if year < 1900 || year > current_year {
            return Err(format!("Year must be between 1900 and {}", current_year).into());
        }

        let date = chrono::NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| "Invalid date".to_string())?; // This will return None if the date is invalid (e.g., February 30th on a non-leap year)

        // Check if the date is in the future
        let today = now.date_naive();
        if date > today {
            return Err("You cannot be born in the future!".into());
        }
    }
    Ok((month, day, year))
}
