use framework::{
    command::{CommandCallbackType, CommandResult, ICommand},
    data::{DefaultPrefix, Prefix},
    extractors::InteractionOptions,
};
use serenity::all::{
    Colour, CommandDataOptionValue, CommandOptionType, CreateCommandOption, CreateEmbed,
};
use utils::BotPermission;

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
    options: InteractionOptions,
    default_prefix: DefaultPrefix,
    prefix: Prefix,
) -> CommandResult<CreateEmbed> {
    let mut embed = CreateEmbed::default();
    embed = embed.color(Colour::BLUE);

    if let Some(sub_cmd) = options.get("set")
        && let CommandDataOptionValue::SubCommand(sub_cmd) = sub_cmd
        && let Some(value) = sub_cmd.first()
        && let CommandDataOptionValue::String(new_prefix) = value.value.clone()
    {
        let description = set_prefix(new_prefix, prefix).await;
        embed = embed.description(description);
    } else if options.get("remove").is_some() {
        let description = remove_prefix(default_prefix, prefix).await;
        embed = embed.description(description);
    } else if options.get("get").is_some() {
        let description = get_prefix(default_prefix, prefix).await;
        embed = embed.description(description);
    }

    Ok(Some(embed))
}

async fn legacy(
    options: Vec<String>,
    default_prefix: DefaultPrefix,
    prefix: Prefix,
) -> CommandResult<CreateEmbed> {
    let mut embed = CreateEmbed::default();
    embed = embed.color(Colour::BLUE);
    let description = match options.first().cloned() {
        Some(value) if value == "remove" => remove_prefix(default_prefix, prefix).await,
        Some(value) => set_prefix(value, prefix).await,
        None => get_prefix(default_prefix, prefix).await,
    };
    embed = embed.description(description);
    Ok(Some(embed))
}

async fn set_prefix(new_prefix: String, Prefix(prefix_ptr): Prefix) -> String {
    if let Some(prefix) = prefix_ptr.make_clone().await {
        if prefix == new_prefix {
            return format!("The prefix is already set to `{}`", new_prefix);
        }
        (prefix_ptr.write().await).clone_from(&Some(new_prefix.clone()));
        return format!("Updated the prefix from `{}` to `{}`", prefix, new_prefix);
    }
    prefix_ptr
        .write()
        .await
        .clone_from(&Some(new_prefix.clone()));
    format!("Set the server prefix to `{}`", new_prefix)
}

async fn remove_prefix(
    DefaultPrefix(default_prefix): DefaultPrefix,
    Prefix(prefix_ptr): Prefix,
) -> String {
    if let Some(prefix) = prefix_ptr.make_clone().await {
        (prefix_ptr.write().await).clone_from(&None);
        return format!(
            "Removed server prefix `{}`. Using default prefix `{}` now.",
            prefix, default_prefix
        );
    }
    "There is no prefix set for this server.".to_string()
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
