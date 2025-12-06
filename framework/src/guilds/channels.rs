use std::collections::HashMap;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serenity::{
    all::{ChannelId, Context, Event, GuildChannel, GuildId, UserId, VoiceState},
    async_trait,
    prelude::TypeMapKey,
};
use utils::{Http, Pointer, error};

use crate::{
    ShardData,
    command::CommandAction,
    extractors::{ContextEventExtractor, EventExtractor, Extractor},
    guilds::HTTPGetter,
};

pub struct Channels(pub Pointer<HashMap<ChannelId, GuildChannel>>);

impl TypeMapKey for Channels {
    type Value = Pointer<HashMap<ChannelId, GuildChannel>>;
}

impl Channels {}

#[async_trait]
impl<T> ContextEventExtractor<T> for Channels
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self> {
        let data = ShardData::get(ctx).await?;
        let guild_id = GuildId::extract_event(ev).await?;
        match data
            .guilds
            .get::<Channels, HashMap<ChannelId, GuildChannel>>(guild_id)
            .await
        {
            Some(channels) => Some(Channels(channels)),
            None => match ctx.http.get_channels(guild_id).await {
                Ok(chs) => {
                    let mut map = HashMap::new();
                    for ch in chs {
                        map.insert(ch.id, ch);
                    }

                    match data
                        .guilds
                        .insert::<Channels, HashMap<ChannelId, GuildChannel>>(guild_id, map)
                        .await
                    {
                        Ok(ptr) => Some(Channels(ptr)),
                        Err(e) => {
                            error!("Failed to insert channels for guild {}: {}", guild_id, e);
                            None
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch channels for guild {}: {}", guild_id, e);
                    None
                }
            },
        }
    }
}

#[async_trait]
impl<T> Extractor<T> for Channels
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract(ctx: &Context, ev: &T, _: &Pointer<utils::Parser>) -> Option<Self> {
        Channels::extract_context_event(ctx, ev).await
    }
}

#[async_trait]
impl super::HTTPGetter<(GuildId, ChannelId), GuildChannel> for Channels {
    async fn fetch(&self, http: &Http, key: (GuildId, ChannelId)) -> Option<GuildChannel> {
        let (guild_id, channel_id) = key;
        match self.0.read().await.get(&channel_id).cloned() {
            Some(channel) => Some(channel),
            None => match guild_id.channels(http).await {
                Ok(channels) => {
                    self.0.set(channels).await;
                    self.0.read().await.get(&channel_id).cloned()
                }
                Err(err) => {
                    error!("Failed to fetch channels from guild {}: {}", guild_id, err);
                    None
                }
            },
        }
    }
}

#[async_trait]
impl super::HTTPGetter<GuildId, HashMap<ChannelId, GuildChannel>> for Channels {
    async fn fetch(&self, http: &Http, key: GuildId) -> Option<HashMap<ChannelId, GuildChannel>> {
        Some(self.0.make_clone().await)
    }
}

#[async_trait]
impl<T> ContextEventExtractor<T> for GuildChannel
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
    ChannelId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self> {
        let guild_id = GuildId::extract_event(ev).await?;
        let channel_id: ChannelId = ChannelId::extract_event(ev).await?;
        let channels = Channels::extract_context_event(ctx, ev).await?;
        channels.fetch(&ctx.http, (guild_id, channel_id)).await
    }
}

#[async_trait]
impl Extractor<Event> for GuildChannel {
    async fn extract(ctx: &Context, ev: &Event, _: &Pointer<utils::Parser>) -> Option<Self> {
        match ev {
            Event::ChannelCreate(create) => Some(create.channel.clone()),
            Event::ChannelUpdate(update) => Some(update.channel.clone()),
            _ => GuildChannel::extract_context_event(ctx, ev).await,
        }
    }
}

#[async_trait]
impl Extractor<CommandAction> for GuildChannel {
    async fn extract(
        ctx: &Context,
        ev: &CommandAction,
        _: &Pointer<utils::Parser>,
    ) -> Option<Self> {
        GuildChannel::extract_context_event(ctx, ev).await
    }
}

pub struct ChannelMembers(pub Pointer<HashMap<ChannelId, Pointer<Vec<UserId>>>>);

impl TypeMapKey for ChannelMembers {
    type Value = Pointer<HashMap<ChannelId, Pointer<Vec<UserId>>>>;
}

impl ChannelMembers {
    pub fn new(voice_state: &HashMap<UserId, VoiceState>) -> Self {
        let mut map: HashMap<ChannelId, Vec<UserId>> = HashMap::new();
        for (user_id, voice_state) in voice_state.iter() {
            if let Some(channel_id) = voice_state.channel_id {
                map.entry(channel_id).or_default().push(*user_id);
            }
        }
        let ptr_map = map
            .par_iter()
            .map(|(channel_id, user_ids)| (*channel_id, Pointer::new(user_ids.clone())))
            .collect();
        ChannelMembers(Pointer::new(ptr_map))
    }
}

#[async_trait]
impl<T> ContextEventExtractor<T> for ChannelMembers
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self> {
        let data = ShardData::get(ctx).await?;
        let guild_id = GuildId::extract_event(ev).await?;
        data.guilds
            .get::<ChannelMembers, HashMap<ChannelId, Pointer<Vec<UserId>>>>(guild_id)
            .await
            .map(Self)
    }
}

#[async_trait]
impl<T> Extractor<T> for ChannelMembers
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract(ctx: &Context, ev: &T, _: &Pointer<utils::Parser>) -> Option<Self> {
        ChannelMembers::extract_context_event(ctx, ev).await
    }
}
