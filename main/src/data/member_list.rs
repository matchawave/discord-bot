use std::collections::HashMap;

use framework::{
    extractors::{ContextEventExtractor, ContextExtractor, EventExtractor, Extractor},
    guilds::Guilds,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serenity::{
    all::{Context, GuildId, Member, UserId},
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};
use utils::{MemberData, Parser, Pointer, error};

type MemberMap = HashMap<UserId, MemberData>;

#[derive(Clone, Default)]
pub struct MemberList(Pointer<MemberMap>);

impl MemberList {
    pub async fn remove(&self, user_id: UserId) -> Option<MemberData> {
        (self.0.write().await).remove(&user_id)
    }

    pub async fn insert(&self, member: &Member) {
        if let Some(join_date) = member.joined_at {
            (self.0.write().await).insert(
                member.user.id,
                MemberData {
                    is_bot: member.user.bot,
                    join_date,
                },
            );
        }
    }

    pub async fn get(&self, user_id: UserId) -> Option<MemberData> {
        (self.0.read().await).get(&user_id).cloned()
    }
    /// Counts the number of human and bot members in the member list.
    ///
    /// Returns a tuple containing the count of human members and bot members.
    pub async fn count(&self) -> (usize, usize) {
        (self.0.read().await.par_iter())
            .fold(
                || (0, 0),
                |(h, b), (_id, data)| {
                    if data.is_bot { (h, b + 1) } else { (h + 1, b) }
                },
            )
            .reduce(|| (0, 0), |(h1, b1), (h2, b2)| (h1 + h2, b1 + b2))
    }

    pub async fn len(&self) -> usize {
        self.0.read().await.len()
    }

    pub async fn debug_print(&self) -> String {
        let data = self.0.read().await;
        if data.is_empty() {
            return "Members Info is empty.".to_string();
        }
        let mut output = String::new();
        output.push_str(&format!("\nTotal Members: {}", data.len()));
        for (user_id, member_data) in data.iter() {
            output.push_str(&format!(
                "\n    {}, Bot: {}, Joined: {}",
                user_id, member_data.is_bot, member_data.join_date
            ));
        }
        output
    }

    pub async fn extract_from_guild(guilds: &Guilds, guild_id: GuildId) -> Option<Self> {
        let guild_map = guilds.get::<MemberList, MemberMap>(guild_id).await?;
        Some(MemberList(guild_map))
    }

    pub async fn from_map(map: &Pointer<TypeMap>) -> Option<Self> {
        let type_map = map.read().await;
        let member_map = type_map.get::<MemberList>()?;
        Some(MemberList(member_map.clone()))
    }
}

impl TypeMapKey for MemberList {
    type Value = Pointer<MemberMap>;
}

#[async_trait]
impl<T> ContextEventExtractor<T> for MemberList
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self> {
        let guild_id = GuildId::extract_event(ev).await?;
        let guilds = Guilds::extract_context(ctx).await?;

        match Self::extract_from_guild(&guilds, guild_id).await {
            Some(members) => Some(members),
            None => {
                let ptr = guilds
                    .insert::<MemberList, MemberMap>(guild_id, HashMap::new())
                    .await;
                ptr.map(Self)
                    .map_err(|e| {
                        error!(
                            "Failed to insert Members data for guild {}: {}",
                            guild_id, e
                        );
                    })
                    .ok()
            }
        }
    }
}

#[async_trait]
impl<T> Extractor<T> for MemberList
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract(ctx: &Context, ev: &T, _: &Pointer<Parser>) -> Option<Self> {
        MemberList::extract_context_event(ctx, ev).await
    }
}
