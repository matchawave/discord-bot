use serenity::{
    all::{Context, Event, GuildId, UserId, VoiceState},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::{cached, command::CommandAction, extractors::Extractor};

cached!(VoiceStates, VoiceState, (GuildId, UserId));

#[async_trait]
impl Extractor<Event> for VoiceState {
    async fn extract(_ctx: &Context, ev: &Event, _p: &Pointer<Parser>) -> Option<Self> {
        match ev {
            Event::VoiceStateUpdate(voice_state_update) => {
                Some(voice_state_update.voice_state.clone())
            }
            _ => None,
        }
    }
}

#[async_trait]
impl Extractor<CommandAction> for VoiceState {
    async fn extract(ctx: &Context, action: &CommandAction, p: &Pointer<Parser>) -> Option<Self> {
        let (guild_id, user_id) = get_ids(ctx, action, p).await?;
        let voice_states = VoiceStates::extract(ctx, action, p).await?;
        fetch_cached(&voice_states, guild_id, user_id).await
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

async fn fetch_cached(
    state: &VoiceStates,
    guild_id: GuildId,
    user_id: UserId,
) -> Option<VoiceState> {
    if let Some(state) = state.get((guild_id, user_id)).await {
        Some(state.make_clone().await)
    } else {
        None
    }
}
