use framework::{
    command::{CommandBuilder, CommandResult, ICommand},
    extractors::InteractionOptions,
    guilds::{HTTPGetter, Members},
};
use serenity::all::{
    Colour, CommandOptionType, CreateCommandOption, CreateEmbed, CreateEmbedAuthor, GuildId,
    Member, UserId,
};
use utils::{HttpType, MemberOption, command_error};

use crate::commands::get_author_embed;

const NAME: &str = "avatar";
const DESCRIPTION: &str = "Get the avatar of a user";

pub fn command() -> ICommand {
    let options = CreateCommandOption::new(
        CommandOptionType::User,
        "user",
        "The user to get the avatar of",
    );
    CommandBuilder::default()
        .options(&[options])
        .slash(interaction)
        .legacy(legacy)
        .build(NAME, DESCRIPTION)
}

async fn interaction(
    http: HttpType,
    options: InteractionOptions,
    members: Members,
    user_id: UserId,
    guild_id: GuildId,
) -> CommandResult<CreateEmbed> {
    let Some(mut target) = (match options.get("user").and_then(|v| v.as_user_id()) {
        Some(id) => members.fetch(&http, (guild_id, id)).await,
        None => members.fetch(&http, (guild_id, user_id)).await,
    }) else {
        return command_error!("No member found for the given ID");
    };

    if target.user.accent_colour.is_none()
        && let Some(member) = members.0.get(&target.user.id).await
        && let Ok(fetched) = http.get_user(target.user.id).await
    {
        member.write().await.user = fetched;
        target = member.make_clone().await;
    }

    let Some(author) = get_author_embed(http, members, &target, user_id).await else {
        return command_error!("Author member not found");
    };

    Ok(Some(create_embed(target, author)))
}

async fn legacy(
    http: HttpType,
    options: Vec<String>,
    members: Members,
    user_id: UserId,
    guild_id: GuildId,
) -> CommandResult<CreateEmbed> {
    let Some(mut target) = (match options.first().map(|id| id.parse::<MemberOption>()) {
        Some(Ok(id)) => members.fetch(&http, (guild_id, id.into())).await,
        Some(Err(e)) => return Err(e),
        None => members.fetch(&http, (guild_id, user_id)).await,
    }) else {
        return command_error!("No member found for the given ID");
    };

    if target.user.accent_colour.is_none()
        && let Some(member) = members.0.get(&target.user.id).await
        && let Ok(fetched) = http.get_user(target.user.id).await
    {
        member.write().await.user = fetched;
        target = member.make_clone().await;
    }

    let Some(author) = get_author_embed(http, members, &target, user_id).await else {
        return command_error!("Author member not found");
    };

    Ok(Some(create_embed(target, author)))
}

fn create_embed(target: Member, author: CreateEmbedAuthor) -> CreateEmbed {
    let color = target.user.accent_colour.unwrap_or(Colour::BLITZ_BLUE);
    let avatar_url = target
        .user
        .avatar_url()
        .unwrap_or(target.user.default_avatar_url());
    CreateEmbed::default()
        .title(format!("Avatar of {}", target.user.tag()))
        .image(avatar_url)
        .color(color)
        .author(author)
}
