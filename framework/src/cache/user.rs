use std::sync::Arc;

use serenity::{
    all::{Context, Event, GuildId, Member, UserId},
    async_trait,
};
use utils::{Parser, Pointer, error};

use crate::{cache::HTTPGetter, cached, command::CommandAction, extractors::Extractor};

cached!(Members, Member, (GuildId, UserId));

#[async_trait]
impl HTTPGetter<(GuildId, UserId), Member> for Members {
    async fn fetch(
        &self,
        http: &Arc<serenity::http::Http>,
        key: (GuildId, UserId),
    ) -> Option<Member> {
        let (guild_id, user_id) = key;
        match self.0.read().await.get(&key).await {
            Some(member) => return Some(member.clone()),
            None => match guild_id.member(http, user_id).await {
                Ok(member) => Some(member),
                Err(err) => {
                    error!(
                        "Failed to fetch member {} from guild {}: {}",
                        user_id, guild_id, err
                    );
                    None
                }
            },
        }
    }
}

#[async_trait]
impl Extractor<Event> for Member {
    async fn extract(ctx: &Context, ev: &Event, p: &Pointer<Parser>) -> Option<Self> {
        let (guild_id, user_id) = get_ids(ctx, ev, p).await?;
        let members = Members::extract(ctx, ev, p).await?;
        members.fetch(&ctx.http, (guild_id, user_id)).await
    }
}

#[async_trait]
impl Extractor<CommandAction> for Member {
    async fn extract(ctx: &Context, action: &CommandAction, p: &Pointer<Parser>) -> Option<Self> {
        let (guild_id, user_id) = get_ids(ctx, action, p).await?;
        let members = Members::extract(ctx, action, p).await?;
        members.fetch(&ctx.http, (guild_id, user_id)).await
    }
}

async fn get_ids<T>(ctx: &Context, ev: &T, p: &Pointer<Parser>) -> Option<(GuildId, UserId)>
where
    GuildId: Extractor<T>,
    UserId: Extractor<T>,
{
    let guild_id = GuildId::extract(ctx, ev, p).await?;
    let user_id = UserId::extract(ctx, ev, p).await?;
    Some((guild_id, user_id))
}
