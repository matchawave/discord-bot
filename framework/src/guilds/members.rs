use std::collections::HashMap;

use moka::future::Cache;
use serenity::{
    all::{Context, Event, GuildId, Member, UserId},
    async_trait,
    prelude::TypeMapKey,
};
use utils::{Http, Parser, Pointer, error};

use crate::{
    ShardData,
    command::CommandAction,
    extractors::{ContextEventExtractor, EventExtractor, Extractor},
    guilds::HTTPGetter,
};

pub struct Members(pub Cache<UserId, Pointer<Member>>);

impl TypeMapKey for Members {
    type Value = Cache<UserId, Pointer<Member>>;
}

impl Members {
    pub async fn new(map: &HashMap<UserId, Member>) -> Self {
        let cache = Cache::builder().build();
        for (user_id, member) in map {
            cache.insert(*user_id, Pointer::new(member.clone())).await;
        }
        Members(cache)
    }
}

#[async_trait]
impl<T> ContextEventExtractor<T> for Members
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self> {
        let data = ShardData::get(ctx).await?;
        let guild_id = GuildId::extract_event(ev).await?;
        let map = data.guilds.map(guild_id).await?;
        map.read().await.get::<Members>().cloned().map(Members)
    }
}

#[async_trait]
impl<T> Extractor<T> for Members
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract(ctx: &Context, ev: &T, _: &Pointer<utils::Parser>) -> Option<Self> {
        Members::extract_context_event(ctx, ev).await
    }
}

#[async_trait]
impl super::HTTPGetter<(GuildId, UserId), Member> for Members {
    async fn fetch(&self, http: &Http, key: (GuildId, UserId)) -> Option<Member> {
        let (guild_id, user_id) = key;
        match self.0.get(&user_id).await {
            Some(m) => Some(m.make_clone().await),
            None => match guild_id.member(http, user_id).await {
                Ok(member) => {
                    self.0.insert(user_id, Pointer::new(member.clone())).await;
                    Some(member)
                }
                Err(err) => {
                    error!(
                        "Failed to fetch member {} in guild {}: {}",
                        user_id, guild_id, err
                    );
                    None
                }
            },
        }
    }
}

#[async_trait]
impl Extractor<Event> for Member
where
    GuildId: EventExtractor<Event>,
    UserId: EventExtractor<Event>,
{
    async fn extract(ctx: &Context, ev: &Event, _: &Pointer<Parser>) -> Option<Self> {
        let guild_id = GuildId::extract_event(ev).await?;
        let user_id = UserId::extract_event(ev).await?;
        let members = Members::extract_context_event(ctx, ev).await?;
        members.fetch(&ctx.http, (guild_id, user_id)).await
    }
}

#[async_trait]
impl Extractor<CommandAction> for Member {
    async fn extract(ctx: &Context, action: &CommandAction, _: &Pointer<Parser>) -> Option<Self> {
        let guild_id = GuildId::extract_event(action).await?;
        let user_id = UserId::extract_event(action).await?;
        let members = Members::extract_context_event(ctx, action).await?;
        members.fetch(&ctx.http, (guild_id, user_id)).await
    }
}
