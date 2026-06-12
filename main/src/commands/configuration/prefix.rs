use framework::{
    command::{CommandBuilder, CommandResult, ICommand},
    data::{DefaultPrefix, Prefix},
    extractors::InteractionOptions,
};
use serenity::{
    all::{
        Colour, CommandDataOptionValue, CommandOptionType, CreateCommandOption, CreateEmbed,
        GuildId,
    },
    model::guild,
};
use utils::{BotPermission, ResponseError, command_error, error};

use crate::global::backend_http::BackendHttp;

const NAME: &str = "prefix";
const DESCRIPTION: &str = "Configure the bot's command prefix";

pub fn command() -> ICommand {
    let set_options = CreateCommandOption::new(
        CommandOptionType::SubCommand,
        "set",
        "Set the server prefix",
    )
    .add_sub_option(
        CreateCommandOption::new(CommandOptionType::String, "value", "The new prefix")
            .required(true),
    );

    let remove_options = CreateCommandOption::new(
        CommandOptionType::SubCommand,
        "remove",
        "Remove the custom server prefix",
    );

    let get_options = CreateCommandOption::new(
        CommandOptionType::SubCommand,
        "get",
        "Get the current server prefix",
    );
    CommandBuilder::default()
        .options(&[set_options, remove_options, get_options])
        .permissions(&[BotPermission::BotMaster])
        .slash(interaction)
        // .legacy(legacy)
        .build(NAME, DESCRIPTION)
}

async fn interaction(
    guild_id: GuildId,
    options: InteractionOptions,
    default_prefix: DefaultPrefix,
    prefix: Prefix,
    backend_http: BackendHttp,
) -> CommandResult<CreateEmbed> {
    let mut embed = CreateEmbed::default();
    embed = embed.color(Colour::BLUE);

    if let Some(sub_cmd) = options.get("set")
        && let CommandDataOptionValue::SubCommand(sub_cmd) = sub_cmd
        && let Some(value) = sub_cmd.first()
        && let CommandDataOptionValue::String(new_prefix) = value.value.clone()
    {
        let description = set_prefix(guild_id, new_prefix, prefix, backend_http).await?;
        embed = embed.description(description);
    } else if options.get("remove").is_some() {
        let description = remove_prefix(guild_id, default_prefix, prefix, backend_http).await?;
        embed = embed.description(description);
    } else if options.get("get").is_some() {
        let description = get_prefix(default_prefix, prefix).await;
        embed = embed.description(description);
    }

    Ok(Some(embed))
}

// async fn legacy(
//     options: Vec<String>,
//     default_prefix: DefaultPrefix,
//     prefix: Prefix,
// ) -> CommandResult<CreateEmbed> {
//     let mut embed = CreateEmbed::default();
//     embed = embed.color(Colour::BLUE);
//     let description = match options.first().cloned() {
//         Some(value) if value == "remove" => remove_prefix(default_prefix, prefix).await,
//         Some(value) => set_prefix(value, prefix).await,
//         None => get_prefix(default_prefix, prefix).await,
//     };
//     embed = embed.description(description);
//     Ok(Some(embed))
// }

async fn set_prefix(
    guild_id: GuildId,
    new_prefix: String,
    Prefix(prefix_ptr): Prefix,
    backend_http: BackendHttp,
) -> Result<String, ResponseError> {
    let old_prefix_ptr = prefix_ptr.make_clone().await;
    if let Some(prefix) = &old_prefix_ptr
        && prefix == &new_prefix
    {
        return Ok(format!("The prefix is already set to `{}`", new_prefix));
    }

    let endpoint = format!("guild/{}/prefix", guild_id);
    (backend_http
        .api()
        .post::<_, ()>(&endpoint, &new_prefix)
        .await)
        .map_err(|e| {
            error!("Guild {guild_id}: Failed to update prefix in backend: {e}",);
            ResponseError::new_silent("Failed to update the prefix. Please try again later.")
        })?;
    (prefix_ptr.write().await).clone_from(&Some(new_prefix.clone()));

    if let Some(prefix) = old_prefix_ptr {
        return Ok(format!(
            "Updated the prefix from `{prefix}` to `{new_prefix}`",
        ));
    }
    Ok(format!("Set the server prefix to `{new_prefix}`"))
}

async fn remove_prefix(
    guild_id: GuildId,
    DefaultPrefix(default_prefix): DefaultPrefix,
    Prefix(prefix_ptr): Prefix,
    backend_http: BackendHttp,
) -> Result<String, ResponseError> {
    let Some(prefix) = prefix_ptr.make_clone().await else {
        return Err(ResponseError::new_silent(
            "There is no prefix set for this server.",
        ));
    };
    let endpoint = format!("guild/{}/prefix", guild_id);
    (backend_http.api().delete::<(), ()>(&endpoint, &()).await).map_err(|e| {
        error!("Guild {guild_id}: Failed to remove prefix in backend: {e}",);
        ResponseError::new_silent("Failed to remove the prefix. Please try again later.")
    })?;
    (prefix_ptr.write().await).clone_from(&None);
    Ok(format!(
        "Removed server prefix `{}`. Using default prefix `{}` now.",
        prefix, default_prefix
    ))
}

async fn get_prefix(
    DefaultPrefix(default_prefix): DefaultPrefix,
    Prefix(prefix_ptr): Prefix,
) -> String {
    if let Some(prefix) = prefix_ptr.make_clone().await {
        return format!("The current prefix is: `{}`", prefix);
    }
    format!("Using default prefix: `{}`", default_prefix)
}
