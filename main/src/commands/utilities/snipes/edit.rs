use framework::{
    command::{CommandCallbackType, CommandResult, ICommand},
    extractors::InteractionOptions,
    guilds::{HTTPGetter, Messages},
};
use serenity::all::{
    ChannelId, Colour, CommandOptionType, CreateAllowedMentions, CreateCommandOption, CreateEmbed,
    CreateMessage, GuildId, Member, Mentionable,
};
use utils::{BotPermission, Http, error};

use crate::{cache::snipe::EditSnipes, no_snipes};

use super::{super::super::author_embed, interaction_index, legacy_index};

const NAME: &str = "editsnipe";
const DESCRIPTION: &str = "Snipe the last editted messages in a channel";

pub fn command() -> ICommand {
    let index_option = CreateCommandOption::new(
        CommandOptionType::Integer,
        "index",
        "The index of the editted message to snipe",
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
    _guild_id: GuildId,
    channel_id: ChannelId,
    snipes: EditSnipes,
    options: InteractionOptions,
    messages: Messages,
    member: Member,
) -> CommandResult<CreateEmbed> {
    let index = interaction_index(options);

    match execute(http, channel_id, snipes, messages, member, index).await {
        Ok(embed) => Ok(embed),
        Err(e) => Err(e),
    }
}

async fn legacy(
    http: Http,
    _guild_id: GuildId,
    channel_id: ChannelId,
    snipes: EditSnipes,
    options: Vec<String>,
    messages: Messages,
    member: Member,
) -> CommandResult<CreateEmbed> {
    let index = legacy_index(options)?;

    match execute(http, channel_id, snipes, messages, member, index).await {
        Ok(embed) => Ok(embed),
        Err(e) => Err(e),
    }
}

async fn execute(
    http: Http,
    channel_id: ChannelId,
    snipes_repo: EditSnipes,
    messages: Messages,
    member: Member,
    index: i64,
) -> CommandResult<CreateEmbed> {
    let Some(snipes) = snipes_repo.0.get(&channel_id).await else {
        return Ok(Some(no_snipes!(
            Colour::BLITZ_BLUE,
            "{}: No **edited messages** found in the past **2 hours**",
            member.user.id.mention().to_string(),
        )));
    };
    let snipe = {
        let snipes_read = snipes.read().await;
        let new_idx = { snipes_read.len() as i64 - 1 - index };
        match snipes_read.get(new_idx as usize) {
            Some(snipe) => snipe.clone(),
            None => {
                return Ok(Some(no_snipes!(
                    Colour::BLITZ_BLUE,
                    "{}: No **edited message** found at index `{}`",
                    member.user.id.mention().to_string(),
                    index,
                )));
            }
        }
    };

    if messages
        .fetch(&http, (channel_id, snipe.id))
        .await
        .is_none()
    {
        if let Some(snipes) = snipes_repo.0.get(&channel_id).await {
            snipes.write().await.remove(index as usize);
        }
        return Ok(Some(no_snipes!(
            Colour::BLITZ_BLUE,
            "{}: The editted message at index `{}` was deleted",
            member.user.id.mention().to_string(),
            index,
        )));
    };

    let mut embed = CreateEmbed::default()
        .colour(Colour::BLITZ_BLUE)
        .description(snipe.content.clone())
        .author(author_embed(&snipe.author));
    if let Some(attachment) = snipe.attachments.first() {
        embed = embed.image(attachment.url.as_str());
    }

    if let Some(ref reference_msg) = snipe.referenced_message {
        if let Err(e) = channel_id
            .send_message(
                http,
                CreateMessage::default()
                    .embed(embed)
                    .reference_message(&**reference_msg)
                    .allowed_mentions(CreateAllowedMentions::default().empty_users()),
            )
            .await
        {
            error!("Failed to send snipe message with reference: {:?}", e);
        }
    } else if let Err(e) = channel_id
        .send_message(http, CreateMessage::default().embed(embed))
        .await
    {
        error!("Failed to send snipe message: {:?}", e);
    }
    Ok(None)
}
