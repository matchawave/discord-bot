#![allow(dead_code)]

use framework::command::{CommandCallbackType as CCT, CommandResult, ICommand};
use serenity::all::{CommandOptionType, CreateCommandOption};
use utils::BotPermission;

const NAME: &str = "example";
const DESCRIPTION: &str = "An example command";

pub fn command() -> ICommand {
    let options = CreateCommandOption::new(CommandOptionType::String, "", "");
    ICommand::new(NAME, DESCRIPTION)
        .options(vec![options])
        .permissions(vec![BotPermission::ManageGuild])
        .callbacks(vec![CCT::slash(interaction), CCT::legacy(legacy)])
}

async fn interaction() -> CommandResult {
    // Handle interaction command
    Ok(None)
}

async fn legacy() {}
