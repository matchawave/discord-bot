use std::sync::Arc;

use dashmap::DashMap;
use serenity::{
    all::{Context, Event, GuildId, PartialGuild},
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};
use utils::{Parser, Pointer};

use crate::{
    command::CommandAction,
    data::{Data, DataExt},
    extractors::Extractor,
};

type GuildMap = DashMap<GuildId, Pointer<PartialGuild>>;
pub struct Guilds(Pointer<GuildMap>);
impl TypeMapKey for Guilds {
    type Value = Pointer<GuildMap>;
}

impl Guilds {
    pub async fn insert(&self, guild: PartialGuild) {
        self.0.write().await.insert(guild.id, Pointer::new(guild));
    }

    pub async fn remove(&self, guild_id: &GuildId) {
        self.0.write().await.remove(guild_id);
    }

    pub async fn get(&self, guild_id: &GuildId) -> Option<Pointer<PartialGuild>> {
        self.0.read().await.get(guild_id).map(|g| g.value().clone())
    }
}

impl DataExt for Guilds {
    fn init(map: &mut TypeMap) {
        map.insert::<Guilds>(Pointer::new(DashMap::new()));
    }

    fn retrieve(map: &Arc<TypeMap>) -> Self {
        map.get::<Guilds>()
            .cloned()
            .map(Guilds)
            .expect("Guilds data not initialized")
    }
}

#[async_trait]
impl Extractor<Event> for Guilds {
    async fn extract(
        ctx: &serenity::all::Context,
        _ev: &Event,
        _p: &Pointer<utils::Parser>,
    ) -> Option<Self> {
        let shard_id = ctx.shard_id;
        let data = Data::get(&ctx.data, shard_id).await?;
        Some(Guilds::retrieve(&data))
    }
}

#[async_trait]
impl Extractor<crate::command::CommandAction> for Guilds {
    async fn extract(
        ctx: &serenity::all::Context,
        _action: &crate::command::CommandAction,
        _p: &Pointer<utils::Parser>,
    ) -> Option<Self> {
        let shard_id = ctx.shard_id;
        let data = Data::get(&ctx.data, shard_id).await?;
        Some(Guilds::retrieve(&data))
    }
}

#[async_trait]
impl Extractor<CommandAction> for Pointer<PartialGuild> {
    async fn extract(ctx: &Context, action: &CommandAction, p: &Pointer<Parser>) -> Option<Self> {
        let guild_id = GuildId::extract(ctx, action, p).await?;
        let Guilds(guilds) = Guilds::extract(ctx, action, p).await?;
        guilds
            .read()
            .await
            .get(&guild_id)
            .map(|pg| pg.value().clone())
    }
}

#[async_trait]
impl Extractor<CommandAction> for PartialGuild {
    async fn extract(ctx: &Context, action: &CommandAction, p: &Pointer<Parser>) -> Option<Self> {
        let guild_id = GuildId::extract(ctx, action, p).await?;
        let Guilds(guilds) = Guilds::extract(ctx, action, p).await?;
        match guilds.read().await.get(&guild_id) {
            Some(guild) => Some(guild.read().await.clone()),
            None => None,
        }
    }
}

#[async_trait]
impl Extractor<Event> for PartialGuild {
    async fn extract(ctx: &Context, ev: &Event, p: &Pointer<Parser>) -> Option<Self> {
        match ev {
            Event::GuildUpdate(guild_update) => Some(guild_update.guild.clone()),
            Event::GuildDelete(guild_delete) => {
                let guild_id = guild_delete.guild.id;
                let Guilds(guilds) = Guilds::extract(ctx, ev, p).await?;
                match guilds.read().await.get(&guild_id) {
                    Some(guild) => Some(guild.read().await.clone()),
                    None => None,
                }
            }
            _ => None,
        }
    }
}
