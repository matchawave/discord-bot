use std::{collections::HashMap, sync::Arc};

use serenity::{
    all::{ChannelId, Context, Event, GuildChannel, GuildId},
    async_trait,
};
use utils::{Parser, Pointer, error};

use crate::{cache::HTTPGetter, cached, command::CommandAction, extractors::Extractor};

cached!(Channels, HashMap<ChannelId, GuildChannel>, GuildId);

#[async_trait]
impl HTTPGetter<GuildId, HashMap<ChannelId, GuildChannel>> for Channels {
    async fn fetch(
        &self,
        http: &Arc<serenity::http::Http>,
        key: GuildId,
    ) -> Option<HashMap<ChannelId, GuildChannel>> {
        match key.channels(http).await {
            Ok(channels) => {
                self.insert(key, channels.clone()).await;
                Some(channels)
            }
            Err(err) => {
                error!("Failed to fetch channels from guild {}: {}", key, err);
                None
            }
        }
    }
}

#[async_trait]
impl Extractor<Event> for GuildChannel {
    async fn extract(ctx: &Context, ev: &Event, p: &Pointer<Parser>) -> Option<Self> {
        let (guild_id, channel_id) = get_ids(ctx, ev, p).await?;
        let channels = Channels::extract(ctx, ev, p).await?;
        channels
            .fetch(&ctx.http, guild_id)
            .await
            .and_then(|map| map.get(&channel_id).cloned())
    }
}

#[async_trait]
impl Extractor<CommandAction> for GuildChannel {
    async fn extract(ctx: &Context, action: &CommandAction, p: &Pointer<Parser>) -> Option<Self> {
        let (guild_id, channel_id) = get_ids(ctx, action, p).await?;
        let channels = Channels::extract(ctx, action, p).await?;
        channels
            .fetch(&ctx.http, guild_id)
            .await
            .and_then(|map| map.get(&channel_id).cloned())
    }
}

async fn get_ids<T>(ctx: &Context, ev: &T, p: &Pointer<Parser>) -> Option<(GuildId, ChannelId)>
where
    GuildId: Extractor<T>,
    ChannelId: Extractor<T>,
{
    let guild_id = GuildId::extract(ctx, ev, p).await?;
    let channel_id = ChannelId::extract(ctx, ev, p).await?;
    Some((guild_id, channel_id))
}
