use std::fmt::Display;

use framework::data::guild::Guilds;
use serenity::all::{
    AfkMetadata, ChannelId, Guild, ImageHash, NsfwLevel, PartialGuild, PremiumTier,
    UnavailableGuild, UserId, VerificationLevel,
};
use utils::info;

use crate::data::member::MembersInfo;

pub async fn create(guild: Guild, guild_members: MembersInfo) {
    info!("Joined guild {} ({})", guild.name, guild.id);
    guild_members.insert(guild.id, Default::default()).await;
}

pub async fn update(guild: PartialGuild, guilds: Guilds) {
    if let Some(old_guild) = guilds.get(&guild.id).await {
        let old_guild = old_guild.read().await.clone();
        let changes = find_differences(&old_guild, &guild);
        changes.iter().for_each(|c| {
            info!("Guild Update for {} ({}): {}", guild.name, guild.id, c);
        });
    }
    guilds.insert(guild).await; // Update the stored guild information
}

pub async fn delete(
    unavailable_guild: UnavailableGuild,
    guild: PartialGuild,
    guild_members: MembersInfo,
) {
    if unavailable_guild.unavailable {
        info!(
            "Guild {} ({}) got deleted (unavailable)",
            guild.name, guild.id
        );
    } else {
        info!("Guild {} ({}) got deleted", guild.name, guild.id);
    }
    guild_members.remove(&guild.id).await;
}

enum GuildChange {
    Name(String, String),
    Icon(Option<ImageHash>, Option<ImageHash>),
    Splash(Option<ImageHash>, Option<ImageHash>),
    DiscoverySplash(Option<ImageHash>, Option<ImageHash>),
    Banner(Option<String>, Option<String>),
    Owner(UserId, UserId),
    Widget(Option<ChannelId>, Option<ChannelId>),
    AfkMetadata(Option<AfkMetadata>, Option<AfkMetadata>),
    VerificationLevel(VerificationLevel, VerificationLevel),
    NsfwLevel(NsfwLevel, NsfwLevel),
    PremiumTier(PremiumTier, PremiumTier),
    VanityUrl(Option<String>, Option<String>),
    Description(Option<String>, Option<String>),
    Features(Vec<String>, Vec<String>),
}

impl Display for GuildChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuildChange::Name(old, new) => write!(f, "Name changed from '{}' to '{}'", old, new),
            GuildChange::Icon(_, _) => write!(f, "Icon changed"),
            GuildChange::Splash(_, _) => write!(f, "Splash changed"),
            GuildChange::DiscoverySplash(_, _) => write!(f, "Discovery splash changed"),
            GuildChange::Banner(_, _) => write!(f, "Banner changed"),
            GuildChange::Owner(old, new) => write!(f, "Owner changed from '{}' to '{}'", old, new),
            GuildChange::Widget(_, _) => write!(f, "Widget channel changed"),
            GuildChange::AfkMetadata(_, _) => write!(f, "AFK metadata changed"),
            GuildChange::VerificationLevel(old, new) => {
                write!(
                    f,
                    "Verification level changed from '{:?}' to '{:?}'",
                    old, new
                )
            }
            GuildChange::NsfwLevel(old, new) => {
                write!(f, "NSFW level changed from '{:?}' to '{:?}'", old, new)
            }
            GuildChange::PremiumTier(old, new) => {
                write!(f, "Premium tier changed from '{:?}' to '{:?}'", old, new)
            }
            GuildChange::VanityUrl(old, new) => {
                write!(f, "Vanity URL changed from '{:?}' to '{:?}'", old, new)
            }
            GuildChange::Description(old, new) => {
                write!(f, "Description changed from '{:?}' to '{:?}'", old, new)
            }
            GuildChange::Features(old, new) => {
                write!(f, "Features changed from '{:?}' to '{:?}'", old, new)
            }
        }
    }
}

fn find_differences(old: &PartialGuild, new: &PartialGuild) -> Vec<GuildChange> {
    let mut diffs = Vec::new();

    if old.name != new.name {
        diffs.push(GuildChange::Name(old.name.clone(), new.name.clone()));
    }
    if old.icon != new.icon {
        diffs.push(GuildChange::Icon(old.icon, new.icon));
    }
    if old.splash != new.splash {
        diffs.push(GuildChange::Splash(old.splash, new.splash));
    }
    if old.discovery_splash != new.discovery_splash {
        diffs.push(GuildChange::DiscoverySplash(
            old.discovery_splash,
            new.discovery_splash,
        ));
    }
    if old.banner != new.banner {
        diffs.push(GuildChange::Banner(old.banner.clone(), new.banner.clone()));
    }
    if old.owner_id != new.owner_id {
        diffs.push(GuildChange::Owner(old.owner_id, new.owner_id));
    }
    if old.afk_metadata != new.afk_metadata {
        diffs.push(GuildChange::AfkMetadata(
            old.afk_metadata.clone(),
            new.afk_metadata.clone(),
        ));
    }
    if old.verification_level != new.verification_level {
        diffs.push(GuildChange::VerificationLevel(
            old.verification_level,
            new.verification_level,
        ));
    }
    if old.widget_channel_id != new.widget_channel_id {
        diffs.push(GuildChange::Widget(
            old.widget_channel_id,
            new.widget_channel_id,
        ));
    }

    if old.nsfw_level != new.nsfw_level {
        diffs.push(GuildChange::NsfwLevel(old.nsfw_level, new.nsfw_level));
    }
    if old.premium_tier != new.premium_tier {
        diffs.push(GuildChange::PremiumTier(old.premium_tier, new.premium_tier));
    }
    if old.vanity_url_code != new.vanity_url_code {
        diffs.push(GuildChange::VanityUrl(
            old.vanity_url_code.clone(),
            new.vanity_url_code.clone(),
        ));
    }
    if old.description != new.description {
        diffs.push(GuildChange::Description(
            old.description.clone(),
            new.description.clone(),
        ));
    }
    if old.features.iter().collect::<Vec<_>>() != new.features.iter().collect::<Vec<_>>() {
        diffs.push(GuildChange::Features(
            old.features.clone(),
            new.features.clone(),
        ));
    }

    diffs
}
