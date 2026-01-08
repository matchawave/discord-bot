use std::{collections::HashMap, sync::Arc};

use framework::{
    command::{CommandCallbackType, CommandResult, ICommand},
    extractors::ShardManagerContainer,
    guilds::Channels,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serenity::all::{
    ChannelId, Colour, CreateEmbed, FormattedTimestamp, FormattedTimestampStyle, GuildChannel,
    Mentionable, PartialGuild, PremiumTier, ShardId, ShardManager, VerificationLevel,
};

use crate::data::member_list::MemberList;

const NAME: &str = "serverinfo";
const DESCRIPTION: &str = "Get information about the server";

pub fn command() -> ICommand {
    ICommand::new(NAME, DESCRIPTION).callbacks(vec![
        CommandCallbackType::slash(interaction),
        CommandCallbackType::legacy(legacy),
    ])
}

async fn interaction(
    ShardManagerContainer(shard_manager): ShardManagerContainer,
    shard_id: ShardId,
    guild: PartialGuild,
    channels: Channels,
    members_list: Option<MemberList>,
) -> CommandResult<CreateEmbed> {
    Ok(Some(
        execute(shard_manager, shard_id, guild, channels, members_list).await,
    ))
}

async fn legacy(
    ShardManagerContainer(shard_manager): ShardManagerContainer,
    shard_id: ShardId,
    guild: PartialGuild,
    channels: Channels,
    members_list: Option<MemberList>,
) -> CommandResult<CreateEmbed> {
    Ok(Some(
        execute(shard_manager, shard_id, guild, channels, members_list).await,
    ))
}

async fn execute(
    shard_manager: Arc<ShardManager>,
    shard_id: ShardId,
    guild: PartialGuild,
    channels: Channels,
    members_list: Option<MemberList>,
) -> CreateEmbed {
    let mut fields = vec![];
    fields.push((
        "Owner".to_string(),
        guild.owner_id.mention().to_string(),
        true,
    ));
    if let Some(m_list) = members_list {
        let members = get_members(m_list).await;
        fields.push(("Members".to_string(), members, true));
    }
    fields.push(("Information".to_string(), get_info(&guild), true));
    if let Some(designs) = get_guild_designs(&guild) {
        fields.push(("Designs".to_string(), designs, true));
    }
    {
        let channels = get_channels(channels.0.make_clone().await);
        let title = format!("Channels ({})", channels.0);
        fields.push((title, channels.1, true));
    }
    fields.push(("Counts".to_string(), get_counts(&guild), true));
    let mut embed = CreateEmbed::default()
        .title(format!("{} ({})", guild.name, guild.id))
        .description(get_description(shard_manager, shard_id, &guild).await)
        .colour(Colour::BLITZ_BLUE)
        .fields(fields);

    if let Some(icon_url) = guild.icon_url() {
        embed = embed.thumbnail(icon_url);
    }

    if let Some(banner) = guild.banner_url() {
        embed = embed.image(banner);
    }
    embed
}

async fn get_description(
    manager: Arc<ShardManager>,
    shard_id: ShardId,
    guild: &PartialGuild,
) -> String {
    let total_shards = manager.runners.lock().await.len();
    let created_at = {
        let timestamp = guild.id.created_at();
        format!(
            "Server created on **{}** ({})",
            timestamp.format("%B %d, %Y"), // format for Month Day, Year
            FormattedTimestamp::new(timestamp, Some(FormattedTimestampStyle::RelativeTime))
        )
    };
    let shard_info = format!(
        "__{}__ is on shard **{}/{}**",
        guild.name,
        shard_id.get() + 1,
        total_shards
    );
    format!("{}\n{}", created_at, shard_info)
}

async fn get_members(members: MemberList) -> String {
    let total = members.len().await;
    let (humans, bots) = members.count().await;

    format!(
        "Total Members: {}\nHumans: {}\nBots: {}",
        total, humans, bots
    )
}

fn get_channels(channels: HashMap<ChannelId, GuildChannel>) -> (usize, String) {
    let count = channels.len();

    let (text_count, voice_count, category_count) = channels
        .par_iter()
        .fold(
            || (0, 0, 0),
            |(t, v, c), (_id, channel)| match channel.kind {
                // t = text, v = voice, c = category
                serenity::all::ChannelType::Text => (t + 1, v, c),
                serenity::all::ChannelType::Voice => (t, v + 1, c),
                serenity::all::ChannelType::Category => (t, v, c + 1),
                _ => (t, v, c),
            },
        )
        .reduce(
            || (0, 0, 0),
            |(t1, v1, c1), (t2, v2, c2)| (t1 + t2, v1 + v2, c1 + c2),
        );

    (
        count,
        format!(
            "Text Channels: {}\nVoice Channels: {}\nCategories: {}",
            text_count, voice_count, category_count
        ),
    )
}

fn get_guild_designs(guild: &PartialGuild) -> Option<String> {
    let mut output = String::new();

    if let Some(splash) = &guild.splash_url() {
        output += &format!("[Splash Image]({})", splash);
    }
    if let Some(banner) = &guild.banner_url() {
        if !output.is_empty() {
            output += "\n";
        }
        output += &format!("[Banner Image]({})", banner);
    }
    if let Some(icon) = &guild.icon_url() {
        if !output.is_empty() {
            output += "\n";
        }
        output += &format!("[Icon Image]({})", icon);
    }
    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

fn get_counts(guild: &PartialGuild) -> String {
    let roles = guild.roles.len();
    let emojis = guild.emojis.len();
    let stickers = guild.stickers.len();

    format!(
        "Roles: {}\nEmojis: {}\nStickers: {}",
        roles, emojis, stickers
    )
}

fn get_info(guild: &PartialGuild) -> String {
    let verification = match guild.verification_level {
        VerificationLevel::Higher => "Highest",
        VerificationLevel::High => "High",
        VerificationLevel::Medium => "Medium",
        VerificationLevel::Low => "Low",
        VerificationLevel::None => "None",
        _ => "Unknown",
    };
    let boost_level = match guild.premium_tier {
        PremiumTier::Tier0 => "None",
        PremiumTier::Tier1 => "1",
        PremiumTier::Tier2 => "2",
        PremiumTier::Tier3 => "3",
        _ => "Unknown",
    };
    let mut output = format!(
        "**Verification Level:** {}\n**Boost Level:** {}",
        verification, boost_level,
    );
    if let Some(count) = guild.premium_subscription_count
        && count > 0
    {
        output.push_str(&format!(" ({})", count));
    }
    if let Some(vanity) = &guild.vanity_url_code {
        output.push_str(&format!("\n**Vanity URL:** dmiscord.gg/{}", vanity));
    }

    output
}
