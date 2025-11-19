use framework::{
    command::{CommandCallbackType, CommandResult, ICommand},
    data::Prefixes,
    extractors::InteractionOptions,
};
use serenity::all::{
    Colour, CommandDataOptionValue, CommandOptionType, CreateCommandOption, CreateEmbed, GuildId,
};
use utils::{BotPermission, Pointer};

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
    ICommand::new(NAME, DESCRIPTION)
        .options(vec![set_options, remove_options, get_options])
        .permissions(vec![BotPermission::ManageGuild])
        .callbacks(vec![
            CommandCallbackType::slash(interaction),
            CommandCallbackType::legacy(legacy),
        ])
}

async fn interaction(
    guild_id: GuildId,
    options: InteractionOptions,
    prefixes: Prefixes,
) -> CommandResult<CreateEmbed> {
    let mut embed = CreateEmbed::default();
    embed = embed.color(Colour::BLUE);

    if let Some(sub_cmd) = options.get("set")
        && let CommandDataOptionValue::SubCommand(sub_cmd) = sub_cmd
        && let Some(value) = sub_cmd.first()
        && let CommandDataOptionValue::String(new_prefix) = &value.value
    {
        let prefix_ptr = prefixes.get_ptr(guild_id).await;
        let description = set_prefix(prefix_ptr, guild_id, new_prefix, prefixes).await;
        embed = embed.description(description);
    } else if options.get("remove").is_some() {
        let prefix_ptr = prefixes.get_ptr(guild_id).await;
        let description = remove_prefix(prefix_ptr, guild_id, prefixes).await;
        embed = embed.description(description);
    } else if options.get("get").is_some() {
        let description = get_prefix(guild_id, prefixes).await;
        embed = embed.description(description);
    }

    Ok(Some(embed))
}

async fn legacy(
    guild_id: GuildId,
    options: Vec<String>,
    prefixes: Prefixes,
) -> CommandResult<CreateEmbed> {
    let mut embed = CreateEmbed::default();
    embed = embed.color(Colour::BLUE);
    let prefix_ptr = prefixes.get_ptr(guild_id).await;
    let description = match options.first() {
        Some(value) if value == "remove" => remove_prefix(prefix_ptr, guild_id, prefixes).await,
        Some(value) => set_prefix(prefix_ptr, guild_id, value, prefixes).await,
        None => get_prefix(guild_id, prefixes).await,
    };
    embed = embed.description(description);
    Ok(Some(embed))
}

async fn set_prefix(
    prefix_ptr: Option<Pointer<String>>,
    guild_id: GuildId,
    new_prefix: &String,
    prefixes: Prefixes,
) -> String {
    if let Some(prefix_ptr) = prefix_ptr {
        let prefix = prefix_ptr.make_clone().await;
        if prefix == *new_prefix {
            format!("The prefix is already set to `{}`", new_prefix)
        } else {
            prefix_ptr.write().await.clone_from(new_prefix);
            format!("Updated the prefix from `{}` to `{}`", prefix, new_prefix)
        }
    } else {
        prefixes.insert(guild_id, new_prefix).await;
        format!("Set the prefix to `{}`", new_prefix)
    }
}

async fn remove_prefix(
    prefix_ptr: Option<Pointer<String>>,
    guild_id: GuildId,
    prefixes: Prefixes,
) -> String {
    if let Some(prefix_ptr) = prefix_ptr {
        let prefix = prefix_ptr.make_clone().await;
        prefixes.remove(guild_id).await;
        format!(
            "Removed the custom prefix `{}`. The default prefix `!` will be used.",
            prefix
        )
    } else {
        "There is no custom prefix set for this server.".to_string()
    }
}

async fn get_prefix(guild_id: GuildId, prefixes: Prefixes) -> String {
    let prefix = prefixes.get(guild_id).await;
    format!("The current prefix is `{}`", prefix)
}
