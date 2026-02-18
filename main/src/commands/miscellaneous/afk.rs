#![allow(dead_code)]

use framework::{
    command::{CommandCallbackType as CCT, CommandResult, ICommand},
    extractors::InteractionOptions,
    global::{GlobalMap, UserGlobalType},
};
use serenity::all::{
    CommandOptionType, CreateCommandOption, CreateEmbed, GuildId, Member, Mentionable,
};
use utils::{BotPermission, ResponseError, command_error, error};

use crate::{
    commands::miscellaneous::afk,
    global::{
        afk::AfkStatus,
        backend_http::{self, BackendHttp},
    },
    success,
};

const NAME: &str = "afk";
const DESCRIPTION: &str = "Set your AFK status with an optional reason";

pub fn command() -> ICommand {
    let per_guild_option = CreateCommandOption::new(
        CommandOptionType::Boolean,
        "per_guild",
        "Configure AFK status for specific guilds",
    );
    let default_reason_option = CreateCommandOption::new(
        CommandOptionType::String,
        "default_reason",
        "The default reason for AFK status",
    );

    let config_subcommand = CreateCommandOption::new(
        CommandOptionType::SubCommand,
        "config",
        "Configure your AFK status",
    )
    .add_sub_option(per_guild_option)
    .add_sub_option(default_reason_option);

    ICommand::new(NAME, DESCRIPTION)
        .options(vec![config_subcommand])
        .permissions(vec![])
        .callbacks(vec![CCT::slash(interaction), CCT::legacy(legacy)])
}

async fn interaction(guild: GuildId, options: InteractionOptions) -> CommandResult {
    // Handle interaction command
    Ok(None)
}

async fn legacy(
    guild_id: GuildId,
    options: Vec<String>,
    member: Member,
    map: GlobalMap<AfkStatus>,
    backend_http: BackendHttp,
) -> CommandResult<CreateEmbed> {
    let user_id = member.user.id;
    let key = UserGlobalType::Guild(guild_id, user_id);
    let reason = if options.is_empty() {
        "AFK".to_string()
    } else {
        options.join(" ")
    };
    let afk_status = (backend_http
        .set_user_afk(user_id, Some(guild_id), reason)
        .await)
        .map_err(|e| {
            error!("Error setting AFK status for user {user_id} in guild {guild_id}: {e}");
            ResponseError::Err(format!("{}: Failed to set AFK status", user_id.mention()))
        })?;
    let reason = afk_status.reason.clone();
    map.insert(key, afk_status).await;
    Ok(Some(success!(
        user_id,
        "You're now AFK with the status: {}",
        reason
    )))
}
