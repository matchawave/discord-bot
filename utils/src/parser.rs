use std::collections::HashMap;

use dashmap::DashMap;
use rayon::{
    iter::{
        IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
    },
    slice::ParallelSliceMut,
};
use regex::{Captures, Regex};
use serenity::all::{
    AfkTimeout, ChannelId, ChannelType, FormattedTimestamp, FormattedTimestampStyle, GuildChannel,
    Member, Mentionable, PartialGuild, PremiumTier, ShardId, Timestamp, UserId,
};

use crate::MemberData;

const NOT_AVAILABLE: &str = "`n/a`";

#[derive(Debug, Clone)]
pub struct Parser {
    shard_id: ShardId,
    guild: Option<PartialGuild>,
    channel: (Option<GuildChannel>, Option<GuildChannel>), // (category, channel)
    member: Option<Member>,
    channels: Option<HashMap<ChannelId, GuildChannel>>,
    members: Option<HashMap<UserId, MemberData>>,

    cache: DashMap<String, Option<String>>,
}

impl Parser {
    pub fn new(shard: ShardId) -> Self {
        Self {
            shard_id: shard,
            guild: None,
            channel: (None, None),
            member: None,
            channels: None,
            members: None,
            cache: DashMap::new(),
        }
    }

    pub fn with_channel(mut self, category: Option<GuildChannel>, channel: GuildChannel) -> Self {
        self.channel = (category, Some(channel));
        self
    }

    pub fn with_member(mut self, member: Member) -> Self {
        self.member = Some(member);
        self
    }

    pub fn with_channels(mut self, channels: HashMap<ChannelId, GuildChannel>) -> Self {
        self.channels = Some(channels);
        self
    }

    pub fn with_members(mut self, members: HashMap<UserId, MemberData>) -> Self {
        self.members = Some(members);
        self
    }

    pub fn with_guild(mut self, guild: PartialGuild) -> Self {
        self.guild = Some(guild);
        self
    }

    fn get(&self, path: &str) -> Option<String> {
        if let Some(cached) = self.cache.get(path) {
            return Some(cached.clone().unwrap_or(NOT_AVAILABLE.to_string()));
        }
        None
    }

    fn handle_user(&self, key: &str) -> Option<String> {
        let member = self.member.as_ref()?;
        let guild = self.guild.as_ref()?;

        let roles = if key.contains("role") || key.contains("color") {
            let mut roles = (&member.roles)
                .into_par_iter()
                .filter_map(|r| guild.roles.get(r).cloned())
                .collect::<Vec<_>>();
            roles.par_sort_by(|a, b| b.position.cmp(&a.position));
            Some(roles)
        } else {
            None
        };

        let position = if key.contains("join")
            && let Some(members) = &self.members
        {
            let mut mems = members
                .into_par_iter()
                .map(|m| (*m.0, *m.1))
                .collect::<Vec<_>>();
            mems.par_sort_by(|a, b| a.1.join_date.cmp(&b.1.join_date));
            mems.par_iter()
                .position_first(|m| m.0 == member.user.id)
                .map(|i| i + 1)
        } else {
            None
        };

        match key {
            "id" => Some(member.user.id.to_string()),
            "name" => Some(member.user.name.clone()),
            "tag" => member.user.discriminator.map(|d| d.to_string()),
            "avatar" => member.user.avatar_url(),
            "guild_avatar" => member.avatar_url(),
            "joined_at" => member.joined_at.map(|ts| {
                FormattedTimestamp::new(ts, Some(FormattedTimestampStyle::ShortDateTime))
                    .to_string()
            }),
            "joined_at_timestamp" => member.joined_at.map(|ts| {
                FormattedTimestamp::new(ts, Some(FormattedTimestampStyle::ShortDateTime))
                    .to_string()
            }),
            "created_at" => Some(
                FormattedTimestamp::new(
                    member.user.created_at(),
                    Some(FormattedTimestampStyle::ShortDateTime),
                )
                .to_string(),
            ),
            "created_at_timestamp" => Some(member.user.created_at().to_string()),
            "display_name" => Some(member.display_name().to_string()),
            "boost" => match member.premium_since {
                Some(_) => Some("Yes".to_string()),
                None => Some("No".to_string()),
            },
            "boost_since" => member.premium_since.map(|ts| ts.to_string()),
            "boost_since_timestamp" => member.premium_since.map(|ts| ts.to_string()),
            "color" => roles.and_then(|r| r.first().map(|r| r.colour.hex())),
            "top_role" => roles.and_then(|r| r.first().map(|r| r.name.clone())),
            "role_list" => roles.map(|r| {
                r.into_par_iter()
                    .map(|r| r.id.mention().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
            "role_text_list" => roles.map(|r| {
                r.into_par_iter()
                    .map(|r| r.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
            "bot" => Some(if member.user.bot { "Yes" } else { "No" }.to_string()),
            "join_position" => position.map(|pos| pos.to_string()),
            "join_position_suffix" => position.map(|pos| {
                let suffix = match pos % 10 {
                    1 if pos % 100 != 11 => "st",
                    2 if pos % 100 != 12 => "nd",
                    3 if pos % 100 != 13 => "rd",
                    _ => "th",
                };
                format!("{}{}", pos, suffix)
            }),
            _ => None,
        }
    }

    fn handle_guild(&self, key: &str) -> Option<String> {
        let guild = self.guild.as_ref()?;
        let channels = self.channels.as_ref();

        match key {
            "id" => Some(guild.id.to_string()),
            "name" => Some(guild.name.clone()),
            "count" => guild.approximate_member_count.map(|c| c.to_string()),
            "shard" => Some(self.shard_id.to_string()),
            "owner" => Some(guild.owner_id.mention().to_string()),
            "owner_id" => Some(guild.owner_id.to_string()),
            "created_at" => Some(
                FormattedTimestamp::new(
                    guild.id.created_at(),
                    Some(FormattedTimestampStyle::ShortDateTime),
                )
                .to_string(),
            ),
            "created_at_timestamp" => Some(guild.id.created_at().to_string()),
            "emoji_count" => Some(guild.emojis.len().to_string()),
            "role_count" => Some(guild.roles.len().to_string()),
            "boost_count" => guild
                .premium_subscription_count
                .map(|count| count.to_string()),
            "boost_tier" => Some(premium_tier(guild.premium_tier)),
            "preferred_locale" => Some(guild.preferred_locale.clone()),
            "key_features" => Some(guild.features.join(", ")),
            "icon" => guild.icon_url().map(|url| url.to_string()),
            "banner" => guild.banner_url().map(|url| url.to_string()),
            "splash" => guild.splash_url().map(|url| url.to_string()),
            "max_presences" => guild.max_presences.map(|count| count.to_string()),
            "max_members" => guild.max_members.map(|count| count.to_string()),
            "max_video_channel_users" => {
                guild.max_video_channel_users.map(|count| count.to_string())
            }
            "afk_timeout" => guild
                .afk_metadata
                .as_ref()
                .map(|meta| afk_timeout(meta.afk_timeout)),
            "afk_channel" => guild
                .afk_metadata
                .as_ref()
                .map(|meta| meta.afk_channel_id.to_string()),
            "vanity" => guild
                .vanity_url_code
                .as_ref()
                .map(|code| format!("https://discord.gg/{}", code)),
            "channels" => channels.map(|c| {
                c.values()
                    .map(|ch| ch.id.mention().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
            "channels_count" => channels.map(|c| c.len().to_string()),
            "text_channels" => channels.map(|c| {
                c.into_par_iter()
                    .filter(|ch| matches!(ch.1.kind, ChannelType::Text | ChannelType::News))
                    .map(|ch| ch.1.id.mention().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
            "text_channels_count" => channels.map(|c| {
                c.into_par_iter()
                    .filter(|ch| matches!(ch.1.kind, ChannelType::Text | ChannelType::News))
                    .count()
                    .to_string()
            }),
            "voice_channels" => channels.map(|c| {
                c.into_par_iter()
                    .filter(|ch| matches!(ch.1.kind, ChannelType::Voice))
                    .map(|ch| ch.1.id.mention().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
            "voice_channels_count" => channels.map(|c| {
                c.into_par_iter()
                    .filter(|ch| matches!(ch.1.kind, ChannelType::Voice))
                    .count()
                    .to_string()
            }),
            "category_channels" => channels.map(|c| {
                c.into_par_iter()
                    .filter(|ch| matches!(ch.1.kind, ChannelType::Category))
                    .map(|ch| ch.1.id.mention().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
            "category_channels_count" => channels.map(|c| {
                c.into_par_iter()
                    .filter(|ch| matches!(ch.1.kind, ChannelType::Category))
                    .count()
                    .to_string()
            }),
            _ => None,
        }
    }

    fn handle_channel(&self, key: &str) -> Option<String> {
        let (p, Some(c)) = &self.channel else {
            return None;
        };

        match key {
            "name" => Some(c.name.clone()),
            "id" => Some(c.id.to_string()),
            "topic" => c.topic.clone(),
            "type" => Some(channel_type(c.kind)),
            "category_id" => p.as_ref().map(|c| c.id.to_string()),
            "category" => p.as_ref().map(|c| c.name.clone()),
            "position" => Some(c.position.to_string()),
            "slowmode" => c.rate_limit_per_user.map(|s| s.to_string()),
            _ => None,
        }
    }

    fn handle_date(&self, key: &str) -> Option<String> {
        let now = chrono::Utc::now();
        let value = match key {
            "now" => Some(
                FormattedTimestamp::new(Timestamp::now(), Some(FormattedTimestampStyle::LongDate))
                    .to_string(),
            ),
            "now_text" => Some(now.format("%B %d, %Y").to_string()),
            "now_short" => Some(
                FormattedTimestamp::new(Timestamp::now(), Some(FormattedTimestampStyle::ShortDate))
                    .to_string(),
            ),
            "now_short_text" => Some(now.format("%Y-%m-%d").to_string()),
            _ => None,
        };
        value.clone()
    }

    fn handle_time(&self, key: &str) -> Option<String> {
        let now = chrono::Utc::now();
        let value = match key {
            "now" => Some(
                FormattedTimestamp::new(Timestamp::now(), Some(FormattedTimestampStyle::LongTime))
                    .to_string(),
            ),
            "now_text" => Some(now.format("%I:%M:%S %p").to_string()),
            "now_military_text" => Some(now.format("%H:%M").to_string()),
            "now_short" => Some(
                FormattedTimestamp::new(Timestamp::now(), Some(FormattedTimestampStyle::ShortTime))
                    .to_string(),
            ),
            "now_short_text" => Some(now.format("%Y-%m-%d").to_string()),
            "now_short_military_text" => Some(now.format("%H:%M:%S").to_string()),
            _ => None,
        };
        value.clone()
    }

    fn handle_level(&self, key: &str) -> Option<String> {
        Some("`n/a`".to_string())
    }
}
pub trait Formatter {
    fn format(&self, parser: &Parser) -> String;
}

impl Formatter for &str {
    fn format(&self, parser: &Parser) -> String {
        let re = Regex::new(r"\{([^}]+)\}").unwrap();
        re.replace_all(self, |caps: &Captures| {
            let path = &caps[1];
            match parser.get(path) {
                Some(value) => value,
                None => {
                    let mut parts = path.split('.');

                    let output = match (parts.next(), parts.next()) {
                        (Some(section), Some(key)) => match section {
                            "user" => parser.handle_user(key),
                            "guild" => parser.handle_guild(key),
                            "channel" => parser.handle_channel(key),
                            "date" => parser.handle_date(key),
                            "time" => parser.handle_time(key),
                            "level" => parser.handle_level(key),
                            _ => None,
                        },
                        (Some(section), None) => match section {
                            "user" => {
                                let member = parser.member.as_ref();
                                member.map(|m| m.user.id.mention().to_string())
                            }
                            "channel" => {
                                let channel = parser.channel.1.as_ref();
                                channel.map(|c| c.id.mention().to_string())
                            }
                            _ => None,
                        },
                        _ => None,
                    };
                    parser.cache.insert(path.to_string(), output.clone());
                    output.unwrap_or(NOT_AVAILABLE.to_string())
                }
            }
        })
        .to_string()
    }
}

impl Formatter for String {
    fn format(&self, parser: &Parser) -> Self {
        self.as_str().format(parser)
    }
}

fn premium_tier(tier: PremiumTier) -> String {
    match tier {
        PremiumTier::Tier0 => "No Level".to_string(),
        PremiumTier::Tier1 => "Tier 1".to_string(),
        PremiumTier::Tier2 => "Tier 2".to_string(),
        PremiumTier::Tier3 => "Tier 3".to_string(),
        _ => "Unknown".to_string(),
    }
}

fn afk_timeout(timeout: AfkTimeout) -> String {
    match timeout {
        AfkTimeout::OneMinute => "1 Minute",
        AfkTimeout::FiveMinutes => "5 Minutes",
        AfkTimeout::FifteenMinutes => "15 Minutes",
        AfkTimeout::ThirtyMinutes => "30 Minutes",
        AfkTimeout::OneHour => "1 Hour",
        _ => "Unknown",
    }
    .to_string()
}

fn channel_type(type_: ChannelType) -> String {
    match type_ {
        ChannelType::Text => "Text",
        ChannelType::Voice => "Voice",
        ChannelType::Category => "Category",
        ChannelType::News => "News",
        ChannelType::Stage => "Stage",
        ChannelType::Directory => "Directory",
        ChannelType::Forum => "Forum",
        _ => "Unknown",
    }
    .to_string()
}
