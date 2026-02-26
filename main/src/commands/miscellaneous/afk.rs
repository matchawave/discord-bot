use framework::{
    command::{CommandBuilder, CommandResult, ICommand},
    extractors::InteractionOptions,
    global::{GlobalCache, GlobalMap},
};
use serde::{Deserialize, Serialize};
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommandOption, CreateEmbed,
    GuildId, Member, Mentionable, UserId,
};
use utils::{Pointer, ResponseError, error, info};

use crate::{
    configs::AfkConfig,
    global::{afk::AfkStatus, backend_http::BackendHttp},
    success,
};

const NAME: &str = "afk";
const DESCRIPTION: &str = "Set your AFK status with an optional reason";

const DEFAULT_REASON: &str = "AFK";

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

    CommandBuilder::default()
        .options(vec![config_subcommand])
        .permissions(vec![])
        .slash(interaction)
        .legacy(legacy)
        .build(NAME, DESCRIPTION)
}

#[derive(Debug, Serialize, Deserialize)]
struct AfkCommandConfig {
    per_guild: Option<bool>,
    default_reason: Option<String>,
}

impl From<&Vec<CommandDataOption>> for AfkCommandConfig {
    fn from(options: &Vec<CommandDataOption>) -> Self {
        let mut config = AfkCommandConfig {
            per_guild: None,
            default_reason: None,
        };

        for option in options {
            match option.name.as_str() {
                "per_guild" => {
                    if let CommandDataOptionValue::Boolean(per_guild) = option.value {
                        config.per_guild = Some(per_guild);
                    }
                }
                "default_reason" => {
                    if let CommandDataOptionValue::String(reason) = &option.value {
                        config.default_reason = Some(reason.clone());
                    }
                }
                _ => {}
            }
        }
        config
    }
}

#[derive(Debug, Deserialize)]
struct AfkConfigResponse {
    old_config: Option<AfkConfig>,
    new_config: AfkConfig,
}

async fn interaction(
    user_id: UserId,
    options: InteractionOptions,
    map: GlobalMap<AfkStatus>,
    cache: GlobalCache<AfkConfig>,
    backend_http: BackendHttp,
) -> CommandResult<CreateEmbed> {
    if let Some(CommandDataOptionValue::SubCommand(config_options)) = options.get("config") {
        let new_config_body = AfkCommandConfig::from(config_options);
        if new_config_body.per_guild.is_none() && new_config_body.default_reason.is_none() {
            return Err(ResponseError::Err(format!(
                "{}: You must provide at least one option to update your AFK configuration.",
                user_id.mention()
            )));
        }
        // For simplicity, we'll just use the default reason in this example
        let path = format!("api/afk/user/config/{}", user_id);
        let Some(result): Option<AfkConfigResponse> =
            (backend_http.post(&path, &new_config_body).await).map_err(|e| {
                error!("Error updating AFK config for user {user_id}: {e}");
                ResponseError::Err(format!(
                    "{}: Failed to update AFK configuration",
                    user_id.mention()
                ))
            })?
        else {
            error!("No response received when updating AFK config for user {user_id}");
            return Err(ResponseError::Err(format!(
                "{}: Failed to update AFK configuration",
                user_id.mention()
            )));
        };

        let old_config = result.old_config;
        let new_config = result.new_config;
        cache.insert(None, user_id, new_config.clone()).await;

        let mut response = String::from("AFK configuration updated successfully!");
        if let Some(per_guild) = new_config_body.per_guild {
            response.push_str(&format!("\n + Per Guild: {}", per_guild));
        }
        if let Some(reason) = new_config_body.default_reason {
            response.push_str(&format!("\n + Default Reason: {}", reason));
        }

        tokio::spawn(clear_user_afk_statuses(
            user_id,
            old_config,
            new_config,
            map.clone(),
            backend_http.clone(),
        ));

        return Ok(Some(success!(user_id, "{}", response)));
    }
    Ok(None)
}

/// Check if per_guild changed to reset AFK Statuses
/// This is so that if a user switches from per_guild=false to per_guild=true, their existing global AFK status doesn't override guild-specific ones, and vice versa
/// Preventing confusion and ensuring expected behavior after changing this setting
async fn clear_user_afk_statuses(
    user_id: UserId,
    old_config: Option<AfkConfig>,
    new_config: AfkConfig,
    map: GlobalMap<AfkStatus>,
    backend_http: BackendHttp,
) {
    if let Some(old_config) = old_config
        && old_config.per_guild != new_config.per_guild
        && map.contains_user(user_id).await
    {
        map.clear_user(user_id).await;
        let path = format!("api/afk/user/{}", user_id);
        if let Err(e) = backend_http.delete::<()>(&path).await {
            error!("Error clearing AFK statuses for user {user_id} after per_guild change: {e}");
        }
        info!(
            "User {user_id} changed per_guild setting, clearing AFK statuses to prevent confusion"
        );
    }
}

#[derive(Debug, Serialize)]
struct NewAfkData {
    guild_id: Option<String>,
    reason: String,
}

async fn legacy(
    guild_id: GuildId,
    options: Vec<String>,
    member: Member,
    map: GlobalMap<AfkStatus>,
    backend_http: BackendHttp,
    cache: GlobalCache<AfkConfig>,
) -> CommandResult<CreateEmbed> {
    let user_id = member.user.id;
    let config = get_user_config(user_id, cache, &backend_http).await?;

    let (per_guild, default_reason) = if let Some(config) = config {
        let c = config.read().await;
        (Some(c.per_guild), c.default_reason.clone())
    } else {
        (None, None)
    };

    let reason = if options.is_empty() {
        default_reason.unwrap_or(DEFAULT_REASON.to_string())
    } else {
        options.join(" ")
    };
    let guild_id = per_guild.and_then(|pg| if pg { Some(guild_id) } else { None });

    let path = format!("api/afk/user/{}", user_id);
    let payload = NewAfkData {
        guild_id: guild_id.map(|g| g.to_string()),
        reason: reason.clone(),
    };

    let Some(afk_status): Option<AfkStatus> =
        backend_http.post(&path, &payload).await.map_err(|e| {
            if let Some(guild_id) = guild_id {
                error!("Error setting AFK status for user {user_id} in guild {guild_id}: {e}");
            } else {
                error!("Error setting global AFK status for user {user_id}: {e}");
            }
            ResponseError::Err(format!("{}: Failed to set AFK status", user_id.mention()))
        })?
    else {
        error!("No response received when setting AFK status for user {user_id}");
        return Err(ResponseError::Err(format!(
            "{}: Failed to set AFK status",
            user_id.mention()
        )));
    };

    let reason = afk_status.reason.clone();
    map.insert(guild_id, user_id, afk_status).await;

    Ok(Some(success!(
        user_id,
        "You're now AFK with the status: {}",
        reason
    )))
}

async fn get_user_config(
    user_id: UserId,
    cache: GlobalCache<AfkConfig>,
    backend_http: &BackendHttp,
) -> CommandResult<Pointer<AfkConfig>> {
    if let Some(config) = cache.get(None, user_id).await {
        Ok(config)
    } else {
        let path = format!("api/afk/user/config/{}", user_id);
        match backend_http.get::<AfkConfig>(&path).await {
            Ok(config) => {
                if let Some(config) = &config {
                    let ptr = cache.insert(None, user_id, config.clone()).await;
                    return Ok(Some(ptr));
                }
                cache.insert_none(None, user_id).await;
                Ok(None)
            }
            Err(e) => {
                error!("Error fetching AFK config for user {user_id}: {e}");
                Err(ResponseError::Err(format!(
                    "{}: Failed to fetch AFK configuration",
                    user_id.mention()
                )))
            }
        }
    }
}
