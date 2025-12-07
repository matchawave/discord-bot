use framework::{
    command::{CommandCallbackType, CommandResult, ICommand},
    extractors::InteractionOptions,
    guilds::{HTTPGetter, Members, Messages},
};
use serenity::all::{
    ChannelId, Colour, CommandOptionType, CreateAllowedMentions, CreateCommandOption, CreateEmbed,
    CreateMessage, GuildId, Member, Mentionable,
};
use utils::{BotPermission, Http, error};

use super::{super::super::author_embed, interaction_index, legacy_index};
use crate::{cache::snipe::ReactionSnipes, no_snipes};

const NAME: &str = "reactsnipe";
const DESCRIPTION: &str = "Snipe the last reaction in a channel";

pub fn command() -> ICommand {
    let index_option = CreateCommandOption::new(
        CommandOptionType::Integer,
        "index",
        "The index of the deleted message to snipe",
    )
    .max_int_value(10)
    .min_int_value(1);
    ICommand::new(NAME, DESCRIPTION)
        .options(vec![index_option])
        .permissions(vec![BotPermission::ManageMessages])
        .callbacks(vec![
            CommandCallbackType::slash(interaction),
            CommandCallbackType::legacy(legacy),
        ])
}
async fn interaction(
    http: Http,
    guild_id: GuildId,
    channel_id: ChannelId,
    snipes: ReactionSnipes,
    options: InteractionOptions,
    member: Member,
    members: Members,
    messages: Messages,
) -> CommandResult<CreateEmbed> {
    let index = interaction_index(options);

    match execute(
        http, guild_id, channel_id, snipes, member, members, messages, index,
    )
    .await
    {
        Ok(embed) => Ok(embed),
        Err(e) => Err(e),
    }
}

async fn legacy(
    http: Http,
    guild_id: GuildId,
    channel_id: ChannelId,
    snipes: ReactionSnipes,
    options: Vec<String>,
    member: Member,
    members: Members,
    messages: Messages,
) -> CommandResult<CreateEmbed> {
    let index = legacy_index(options)?;

    match execute(
        http, guild_id, channel_id, snipes, member, members, messages, index,
    )
    .await
    {
        Ok(embed) => Ok(embed),
        Err(e) => Err(e),
    }
}
async fn execute(
    http: Http,
    guild_id: GuildId,
    channel_id: ChannelId,
    snipes: ReactionSnipes,
    member: Member,
    members: Members,
    messages: Messages,
    index: i64,
) -> CommandResult<CreateEmbed> {
    let Some(snipes) = snipes.0.get(&channel_id).await else {
        return Ok(Some(no_snipes!(
            Colour::BLITZ_BLUE,
            "{}: No **removed reactions** found in the past **2 hours**",
            member.user.id.mention().to_string(),
        )));
    };
    let snipe = {
        let snipes_read = snipes.read().await;
        let index = { snipes_read.len() as i64 - 1 - index };
        match snipes_read.get(index as usize) {
            Some(snipe) => snipe.clone(),
            None => {
                return Ok(Some(no_snipes!(
                    Colour::BLITZ_BLUE,
                    "{}: No **reaction** found at index `{}`",
                    member.user.id.mention().to_string(),
                    index,
                )));
            }
        }
    };

    if let Some(user_id) = snipe.user_id
        && let Some(target_member) = members.fetch(&http, (guild_id, user_id)).await
        && let Some(target_message) = messages.fetch(&http, (channel_id, snipe.message_id)).await
    {
        let emoji = snipe.emoji.to_string();
        let embed = CreateEmbed::default()
            .colour(Colour::BLITZ_BLUE)
            .description(format!("{} reacted with {}", user_id.mention(), emoji))
            .author(author_embed(&target_member.user));

        if let Err(e) = channel_id
            .send_message(
                http,
                CreateMessage::default()
                    .embed(embed)
                    .reference_message(&target_message)
                    .allowed_mentions(CreateAllowedMentions::default().empty_users()),
            )
            .await
        {
            error!("Failed to send snipe message with reference: {:?}", e);
        }
    }

    Ok(None)
}
