use framework::{command::ICommand, extractors::InteractionOptions};
use serenity::all::{Colour, CreateEmbed};
use utils::{IntegerOption, ResponseError};
mod edit;
mod reaction;
mod snipe;

pub fn register() -> Vec<ICommand> {
    vec![snipe::command(), edit::command(), reaction::command()]
}

pub fn no_snipe_embed<T: Into<String>>(text: T, c: Colour) -> CreateEmbed {
    CreateEmbed::default().description(text.into()).colour(c)
}
pub fn legacy_index(options: Vec<String>) -> Result<i64, ResponseError> {
    match options.first() {
        Some(s) => match s.parse::<IntegerOption>() {
            Ok(n) => Ok(*n - 1),
            Err(e) => Err(e),
        },
        None => Ok(0),
    }
}

pub fn interaction_index(options: InteractionOptions) -> i64 {
    options.get("index").and_then(|o| o.as_i64()).unwrap_or(1) - 1
}

#[macro_export]
macro_rules! no_snipes {
    () => {
        serenity::all::CreateEmbed::default()
            .description("There are no snipes for this channel.")
            .colour(Colour::BLITZ_BLUE)
    };
    ($c:expr) => {
        serenity::all::CreateEmbed::default()
            .description("There are no snipes for this channel.")
            .colour($c)
    };
    ($c:expr, $($arg:tt)*) => {
        serenity::all::CreateEmbed::default()
            .description(format!($($arg)*))
            .colour($c)
    };
}
