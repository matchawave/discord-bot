use std::collections::HashMap;

use moka::future::Cache;
use serenity::{
    all::{Context, Event, GuildId, UserId, VoiceState},
    async_trait,
    prelude::TypeMapKey,
};
use utils::{Parser, Pointer};

use crate::{
    ShardData,
    extractors::{ContextEventExtractor, EventExtractor, Extractor},
};

pub struct VoiceStates(pub Cache<UserId, VoiceState>);

impl TypeMapKey for VoiceStates {
    type Value = Cache<UserId, VoiceState>;
}

impl Default for VoiceStates {
    fn default() -> Self {
        let cache = Cache::builder().max_capacity(100_000).build();
        Self(cache)
    }
}
impl VoiceStates {
    pub async fn from_voice_states(&self, voice_states: &HashMap<UserId, VoiceState>) {
        for (user_id, voice_state) in voice_states.iter() {
            self.0.insert(*user_id, voice_state.clone()).await;
        }
    }
}

#[async_trait]
impl<T> ContextEventExtractor<T> for VoiceStates
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self> {
        let data = ShardData::get(ctx.shard_id, &ctx.data).await?;
        let guild_id = GuildId::extract_event(ev).await?;
        let map = data.guilds.map(guild_id).await?;
        (map.read().await.get::<VoiceStates>())
            .cloned()
            .map(VoiceStates)
    }
}

#[async_trait]
impl<T> Extractor<T> for VoiceStates
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract(ctx: &Context, ev: &T, _: &Pointer<utils::Parser>) -> Option<Self> {
        VoiceStates::extract_context_event(ctx, ev).await
    }
}

#[async_trait]
impl EventExtractor<Event> for VoiceState {
    async fn extract_event(ev: &Event) -> Option<Self> {
        if let Event::VoiceStateUpdate(ev_state) = ev {
            return Some(ev_state.voice_state.clone());
        }
        None
    }
}

#[async_trait]
impl Extractor<Event> for VoiceState {
    async fn extract(_: &Context, ev: &Event, _: &Pointer<Parser>) -> Option<Self> {
        VoiceState::extract_event(ev).await
    }
}
