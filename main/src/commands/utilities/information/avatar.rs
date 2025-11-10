use std::option;

use framework::{
    cache::{HTTPGetter, Members},
    command::{CommandCallbackType, CommandResult, ICommand},
    data::guild,
    extractors::InteractionOptions,
};
use serenity::all::{
    Colour, CommandOptionType, CreateCommandOption, CreateEmbed, GuildId, Member, UserId,
};
use utils::{BotPermission, Http, MemberOption, command_error};

const NAME: &str = "avatar";
const DESCRIPTION: &str = "Get the avatar of a user";

pub fn command() -> ICommand {
    let options = CreateCommandOption::new(
        CommandOptionType::User,
        "user",
        "The user to get the avatar of",
    );
    ICommand::new(NAME, DESCRIPTION)
        .options(vec![options])
        .callbacks(vec![
            CommandCallbackType::slash(interaction),
            CommandCallbackType::legacy(legacy),
        ])
}

async fn interaction(
    http: Http,
    options: InteractionOptions,
    members: Members,
    user_id: UserId,
    guild_id: GuildId,
) -> CommandResult<CreateEmbed> {
    let Some(target) = (match options.get("user").and_then(|v| v.as_user_id()) {
        Some(id) => members.fetch(&http, (guild_id, id)).await,
        None => members.fetch(&http, (guild_id, user_id)).await,
    }) else {
        return command_error!("No member found for the given ID");
    };
    Ok(Some(create_embed(target)))
}

async fn legacy(
    http: Http,
    options: Vec<String>,
    members: Members,
    user_id: UserId,
    guild_id: GuildId,
) -> CommandResult<CreateEmbed> {
    let Some(target) = (match options.first().map(|id| id.parse::<MemberOption>()) {
        Some(Ok(id)) => members.fetch(&http, (guild_id, id.into())).await,
        Some(Err(e)) => return Err(e),
        None => members.fetch(&http, (guild_id, user_id)).await,
    }) else {
        return command_error!("No member found for the given ID");
    };

    Ok(Some(create_embed(target)))
}

fn create_embed(target: Member) -> CreateEmbed {
    let color = target.user.accent_colour.unwrap_or(Colour::BLITZ_BLUE);
    let avatar_url = target
        .user
        .avatar_url()
        .unwrap_or(target.user.default_avatar_url());
    CreateEmbed::default()
        .title(format!("Avatar of {}", target.user.tag()))
        .image(avatar_url)
        .color(color)
}
