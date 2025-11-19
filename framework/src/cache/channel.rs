use std::{collections::HashMap, sync::Arc};

use serenity::{
    all::{ChannelId, Context, Event, GuildChannel, GuildId},
    async_trait,
};
use utils::{Parser, Pointer, error};

use crate::{cache::HTTPGetter, cached, command::CommandAction, extractors::Extractor};

cached!(Channels, HashMap<ChannelId, GuildChannel>, GuildId);

#[async_trait]
impl HTTPGetter<GuildId, Pointer<HashMap<ChannelId, GuildChannel>>> for Channels {
    async fn fetch(
        &self,
        http: &Arc<serenity::http::Http>,
        key: GuildId,
    ) -> Option<Pointer<HashMap<ChannelId, GuildChannel>>> {
        match self.get(key).await {
            Some(channels) => Some(channels),
            None => match key.channels(http).await {
                Ok(channels) => {
                    self.insert(key, channels.clone()).await;
                    self.get(key).await
                }
                Err(err) => {
                    error!("Failed to fetch channels from guild {}: {}", key, err);
                    None
                }
            },
        }
    }
}

#[async_trait]
impl HTTPGetter<(GuildId, ChannelId), GuildChannel> for Channels {
    async fn fetch(
        &self,
        http: &Arc<serenity::http::Http>,
        key: (GuildId, ChannelId),
    ) -> Option<GuildChannel> {
        if let Some(channels) = self.get(key.0).await {
            return channels.read().await.get(&key.1).cloned();
        }
        match key.0.channels(http).await {
            Ok(channels) => {
                self.insert(key.0, channels).await;
                match self.get(key.0).await {
                    Some(channels) => channels.read().await.get(&key.1).cloned(),
                    None => None,
                }
            }
            Err(err) => {
                error!("Failed to fetch channels from guild {}: {}", key.0, err);
                None
            }
        }
    }
}

#[async_trait]
impl Extractor<Event> for GuildChannel {
    async fn extract(ctx: &Context, ev: &Event, p: &Pointer<Parser>) -> Option<Self> {
        match ev {
            Event::ChannelUpdate(channel) => {
                return Some(channel.channel.clone());
            }
            Event::ChannelCreate(channel) => {
                return Some(channel.channel.clone());
            }
            _ => {
                let channels = Channels::extract(ctx, ev, p).await?;
                let (guild_id, channel_id) = get_ids(ctx, ev, p).await?;
                match channels.fetch(&ctx.http, guild_id).await {
                    Some(map) => map.read().await.get(&channel_id).cloned(),
                    None => None,
                }
            }
        }
    }
}

#[async_trait]
impl Extractor<CommandAction> for GuildChannel {
    async fn extract(ctx: &Context, action: &CommandAction, p: &Pointer<Parser>) -> Option<Self> {
        let (guild_id, channel_id) = get_ids(ctx, action, p).await?;
        let channels = Channels::extract(ctx, action, p).await?;
        match channels.fetch(&ctx.http, guild_id).await {
            Some(map) => map.read().await.get(&channel_id).cloned(),
            None => None,
        }
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
