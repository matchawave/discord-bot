use std::collections::HashMap;

use dashmap::DashMap;
use serenity::{
    all::{Context, Event, GuildId},
    prelude::{TypeMap, TypeMapKey},
};
use utils::{Parser, Pointer, info};

use crate::{command::CommandAction, data::Data, extractors::Extractor};

type CommandAliasesMap = Pointer<DashMap<GuildId, GuildCommandAliases>>;
type GuildCommandAliases = Pointer<HashMap<String, String>>;
pub struct CommandAliases(pub GuildCommandAliases);

impl TypeMapKey for CommandAliases {
    type Value = CommandAliasesMap;
}

impl CommandAliases {
    pub async fn set(data: &Pointer<TypeMap>) {
        data.write()
            .await
            .insert::<CommandAliases>(Pointer::new(DashMap::new()));
    }

    pub async fn get(data: &Pointer<TypeMap>) -> Option<CommandAliasesMap> {
        data.read().await.get::<CommandAliases>().cloned()
    }
}

#[serenity::async_trait]
impl Extractor<Event> for CommandAliases {
    async fn extract(ctx: &Context, action: &Event, p: &Pointer<Parser>) -> Option<Self> {
        let guild_id = GuildId::extract(ctx, action, p).await?;
        let Data(data) = Data::extract(ctx, action, p).await?;

        let aliases = data.get::<CommandAliases>()?.clone();
        let guild_aliases = get_map(aliases, guild_id).await;
        Some(CommandAliases(guild_aliases))
    }
}

#[serenity::async_trait]
impl Extractor<CommandAction> for CommandAliases {
    async fn extract(ctx: &Context, action: &CommandAction, p: &Pointer<Parser>) -> Option<Self> {
        let guild_id = GuildId::extract(ctx, action, p).await?;
        let Data(data) = Data::extract(ctx, action, p).await?;

        let aliases = data.get::<CommandAliases>()?.clone();
        let guild_aliases = get_map(aliases, guild_id).await;
        Some(CommandAliases(guild_aliases))
    }
}

async fn get_map(aliases: CommandAliasesMap, guild_id: GuildId) -> GuildCommandAliases {
    let key_exists = { aliases.read().await.contains_key(&guild_id) };
    if key_exists {
        aliases.read().await.get(&guild_id).unwrap().clone()
    } else {
        info!("Creating new command aliases map for guild {}", guild_id);
        let new_map = Pointer::new(HashMap::new());
        aliases.write().await.insert(guild_id, new_map.clone());
        new_map
    }
}
