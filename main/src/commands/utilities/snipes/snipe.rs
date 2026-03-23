use framework::{
    command::{CommandBuilder, CommandResult, ICommand},
    extractors::InteractionOptions,
};
use serenity::all::{
    ChannelId, Colour, CommandOptionType, CreateAllowedMentions, CreateCommandOption, CreateEmbed,
    CreateMessage, GuildId, Member, Mentionable,
};
use utils::{BotPermission, HttpType, error};

use crate::{cache::snipe::Snipes, no_snipes};

use super::{super::super::author_embed, interaction_index, legacy_index};

const NAME: &str = "snipe";
const DESCRIPTION: &str = "Snipe the last deleted messages in a channel";

pub fn command() -> ICommand {
    let index_option = CreateCommandOption::new(
        CommandOptionType::Integer,
        "index",
        "The index of the deleted message to snipe",
    )
    .max_int_value(10)
    .min_int_value(1);
    CommandBuilder::default()
        .options(&[index_option])
        .permissions(&[BotPermission::ManageMessages])
        .slash(interaction)
        .legacy(legacy)
        .build(NAME, DESCRIPTION)
}

async fn interaction(
    http: HttpType,
    _guild_id: GuildId,
    channel_id: ChannelId,
    snipes: Snipes,
    options: InteractionOptions,
    member: Member,
) -> CommandResult<CreateEmbed> {
    let index = interaction_index(options);

    match execute(http, channel_id, snipes, member, index).await {
        Ok(embed) => Ok(embed),
        Err(e) => Err(e),
    }
}

async fn legacy(
    http: HttpType,
    _guild_id: GuildId,
    channel_id: ChannelId,
    snipes: Snipes,
    options: Vec<String>,
    member: Member,
) -> CommandResult<CreateEmbed> {
    let index = legacy_index(options)?;

    match execute(http, channel_id, snipes, member, index).await {
        Ok(embed) => Ok(embed),
        Err(e) => Err(e),
    }
}

async fn execute(
    http: HttpType,
    channel_id: ChannelId,
    snipes: Snipes,
    member: Member,
    index: i64,
) -> CommandResult<CreateEmbed> {
    let Some(snipes) = snipes.0.get(&channel_id).await else {
        return Ok(Some(no_snipes!(
            Colour::BLITZ_BLUE,
            "{}: No **deleted messages** found in the past **2 hours**",
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
                    "{}: No **deleted message** found at index `{}`",
                    member.user.id.mention().to_string(),
                    index,
                )));
            }
        }
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
