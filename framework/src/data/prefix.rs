use std::{collections::HashMap, sync::Arc};

use serenity::{
    all::{Context, GuildId},
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};
use utils::{Parser, Pointer};

use crate::{DataExtractable, data::Data, extractors::Extractor};

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum ServerPrefix {
    Guild(GuildId),
    Default,
}

#[derive(Clone)]
pub struct Prefixes(Pointer<HashMap<ServerPrefix, Pointer<String>>>);

impl TypeMapKey for Prefixes {
    type Value = Pointer<HashMap<ServerPrefix, Pointer<String>>>;
}

impl Prefixes {
    pub async fn get(&self, guild_id: GuildId) -> String {
        let map = self.0.read().await;
        if let Some(prefix) = map.get(&ServerPrefix::Guild(guild_id)) {
            return prefix.read().await.clone();
        }
        if let Some(prefix) = map.get(&ServerPrefix::Default) {
            return prefix.read().await.clone();
        }
        "!".to_string()
    }

    pub async fn get_ptr(&self, guild_id: GuildId) -> Option<Pointer<String>> {
        let map = self.0.read().await;
        map.get(&ServerPrefix::Guild(guild_id)).cloned()
    }

    pub async fn insert<T: Into<String>>(&self, guild_id: GuildId, prefix: T) {
        let prefix = prefix.into();
        self.0
            .write()
            .await
            .insert(ServerPrefix::Guild(guild_id), Pointer::new(prefix));
    }

    pub async fn remove(&self, guild_id: GuildId) {
        self.0.write().await.remove(&ServerPrefix::Guild(guild_id));
    }
}

impl DataExtractable for Prefixes {
    fn init(map: &mut TypeMap) {
        let mut prefixes = HashMap::new();
        prefixes.insert(ServerPrefix::Default, Pointer::new("!".to_string()));
        map.insert::<Prefixes>(Pointer::new(prefixes));
    }
    fn retrieve(map: &Arc<TypeMap>) -> Option<Self> {
        map.get::<Prefixes>().cloned().map(Self)
    }
}

#[async_trait]
impl<T> Extractor<T> for Prefixes {
    async fn extract(ctx: &Context, _: &T, _: &Pointer<Parser>) -> Option<Self> {
        let data = Data::get(&ctx.data, ctx.shard_id).await?;
        Prefixes::retrieve(&data)
    }
}
