use std::{collections::HashMap, sync::Arc};

use serenity::{
    all::{ChannelId, Context, Event, GuildId, ShardId, UserId},
    async_trait,
};
use utils::{DiscordEvent, Http, Pointer, info};

use crate::{
    build_process,
    extractors::{ContextExtractor, Extractor},
    processes::{ProcessLoop, ProcessManager},
};
use std::{fmt::Debug, hash::Hash};

build_process!(Cooldowns, HashMap<Cooldown, std::time::Instant>);

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Cooldown {
    Command(GuildId, UserId, String),
    Event(GuildId, UserId, DiscordEvent),
    VoiceMaster(GuildId, ChannelId, UserId),
}

impl Cooldown {
    pub(crate) fn guild(&self) -> Option<GuildId> {
        match self {
            Cooldown::Command(guild_id, _, _) => Some(*guild_id),
            Cooldown::Event(guild_id, _, _) => Some(*guild_id),
            Cooldown::VoiceMaster(guild_id, _, _) => Some(*guild_id),
            // _ => None,
        }
    }
    pub(crate) fn user(&self) -> Option<UserId> {
        match self {
            Cooldown::Command(_, user_id, _) => Some(*user_id),
            Cooldown::Event(_, user_id, _) => Some(*user_id),
            Cooldown::VoiceMaster(_, _, user_id) => Some(*user_id),
            // _ => None,
        }
    }
    pub(crate) fn identifier(&self) -> String {
        match self {
            Cooldown::Command(_, _, command_name) => command_name.clone(),
            Cooldown::Event(_, _, identifier) => format!("{identifier:?}"),
            Cooldown::VoiceMaster(_, channel_id, _) => channel_id.to_string(),
        }
    }
}

#[async_trait]
impl ContextExtractor for Arc<Cooldowns> {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        let p_manager = Arc::<ProcessManager>::extract_context(ctx).await?;
        p_manager.get::<Cooldowns>()
    }
}

#[async_trait]
impl<T> Extractor<T> for Arc<Cooldowns>
where
    T: Send + Sync + 'static,
{
    async fn extract(ctx: &Context, _: &T, _: &Pointer<utils::Parser>) -> Option<Self> {
        Arc::<Cooldowns>::extract_context(ctx).await
    }
}

#[async_trait]
impl ProcessLoop for Cooldowns {
    async fn process(&self, _http: Http) {
        loop {
            let now = std::time::Instant::now();
            let map = self.0.read().await.clone();
            for (key, &time) in map.iter() {
                if now > time {
                    match (key.guild(), key.user()) {
                        (Some(guild_id), Some(user_id)) => {
                            info!(
                                "(cooldowns) Expired cooldown for user {} in guild {} for identifier: {}",
                                user_id,
                                guild_id,
                                key.identifier()
                            );
                        }
                        _ => {
                            info!(
                                "(cooldowns) Expired cooldown for identifier: {}",
                                key.identifier()
                            );
                        }
                    }
                    let mut map = self.0.write().await;
                    map.remove(key);
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}
