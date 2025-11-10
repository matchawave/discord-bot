use std::sync::Arc;

use dashmap::DashMap;
use serenity::{
    all::{Context, Event, GuildId},
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};
use utils::{DataType, Parser, Pointer};

use crate::{command::CommandAction, extractors::Extractor};

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum ServerPrefix {
    Guild(GuildId),
    Default,
}

pub struct ServerPrefixes;
pub type ServerPrefixesMap = DashMap<ServerPrefix, String>;
impl TypeMapKey for ServerPrefixes {
    type Value = Arc<ServerPrefixesMap>;
}

pub struct Prefix(pub String);

impl Prefix {
    pub fn set(data: &mut TypeMap) {
        let prefixes = DashMap::new();
        prefixes.insert(ServerPrefix::Default, "!".to_string());
        data.insert::<ServerPrefixes>(Arc::new(prefixes));
    }

    pub async fn get(data: &DataType, guild_id: GuildId) -> Option<Self> {
        let data = data.read().await;
        let prefixes = data.get::<ServerPrefixes>().unwrap();

        prefixes
            .get(&ServerPrefix::Guild(guild_id))
            .or(prefixes.get(&ServerPrefix::Default))
            .map(|v| v.value().clone())
            .map(Self)
    }

    pub async fn update<T: Into<String>>(data: &DataType, guild_id: GuildId, prefix: T) {
        let data = data.read().await;
        if let Some(prefixes) = data.get::<ServerPrefixes>() {
            let prefix = prefix.into();
            if prefix.is_empty() {
                prefixes.remove(&ServerPrefix::Guild(guild_id));
            } else {
                prefixes.insert(ServerPrefix::Guild(guild_id), prefix);
            }
        }
    }
}

#[async_trait]
impl Extractor<Event> for Prefix {
    async fn extract(ctx: &Context, ev: &Event, p: &Pointer<Parser>) -> Option<Self> {
        let guild_id = GuildId::extract(ctx, ev, p).await?;
        Prefix::get(&ctx.data, guild_id).await
    }
}

#[async_trait]
impl Extractor<CommandAction> for Prefix {
    async fn extract(ctx: &Context, ev: &CommandAction, p: &Pointer<Parser>) -> Option<Self> {
        let guild_id = GuildId::extract(ctx, ev, p).await?;
        Prefix::get(&ctx.data, guild_id).await
    }
}
