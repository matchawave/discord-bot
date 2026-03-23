#![allow(dead_code)]

use framework::command::{CommandBuilder, CommandResult, ICommand};
use serenity::all::{CommandOptionType, CreateCommandOption};
use utils::BotPermission;

const NAME: &str = "example";
const DESCRIPTION: &str = "An example command";

pub fn command() -> ICommand {
    let options = CreateCommandOption::new(CommandOptionType::String, "", "");
    CommandBuilder::default()
        .options(&[options])
        .permissions(&[BotPermission::ManageGuild])
        .slash(interaction)
        .legacy(legacy)
        .build(NAME, DESCRIPTION)
}

async fn interaction() -> CommandResult {
    // Handle interaction command
    Ok(None)
}

async fn legacy() {}
