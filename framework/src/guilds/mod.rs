mod channels;
mod members;
mod messages;
mod voice_state;

pub use channels::{ChannelMembers, Channels};
pub use members::Members;
pub use messages::Messages;
use moka::future::Cache;
pub use voice_state::VoiceStates;

use std::{collections::HashMap, hash::Hash};

use macros::DataExtractable;
use serenity::{
    all::{Context, Event, Guild, GuildId, PartialGuild},
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};
use utils::{Http, Parser, Pointer, error};

use crate::{
    Extractable, ShardData,
    command::CommandAction,
    data::Prefix,
    extractors::{ContextEventExtractor, ContextExtractor, EventExtractor, Extractor},
};
pub type GuildMap = Pointer<TypeMap>;

#[derive(Clone, Default, DataExtractable)]
pub struct Guilds(Pointer<HashMap<GuildId, GuildMap>>);

impl Guilds {
    pub async fn new_guild(&self, guild: Guild) {
        let guild_id = guild.id;
        let mut new_map = TypeMap::new();
        let voice_states = VoiceStates::default();
        voice_states.from_voice_states(&guild.voice_states).await;

        new_map.insert::<Channels>(Pointer::new(guild.channels.clone()));
        new_map.insert::<Members>(Members::new(&guild.members).await.0);
        new_map.insert::<Messages>(Messages::default().0);
        new_map.insert::<VoiceStates>(voice_states.0);
        new_map.insert::<ChannelMembers>(ChannelMembers::new(&guild.voice_states).0);
        new_map.insert::<Pointer<PartialGuild>>(Pointer::new(guild.into()));
        new_map.insert::<Prefix>(Pointer::new(None));

        self.0.write().await.insert(guild_id, Pointer::new(new_map));
    }

    pub async fn remove_guild(&self, guild_id: GuildId) -> Option<PartialGuild> {
        if let Some(guild_map) = self.0.write().await.remove(&guild_id) {
            let ptr = (guild_map.read().await)
                .get::<Pointer<PartialGuild>>()
                .cloned();
            if let Some(guild_ptr) = ptr {
                return Some(guild_ptr.make_clone().await);
            }
            error!(
                "Tried to remove guild data from guild data map, but PartialGuild not found: {}",
                guild_id
            );
        }
        error!(
            "Tried to remove guild data from non-existing guild data map: {}",
            guild_id
        );
        None
    }

    pub async fn get_ptr<T>(&self, guild_id: GuildId) -> Option<Pointer<T>>
    where
        T: Send + Sync + 'static,
    {
        let map_read = self.0.read().await;
        let map = map_read.get(&guild_id)?;
        map.read().await.get::<Pointer<T>>().cloned()
    }

    pub async fn get<T, V>(&self, guild_id: GuildId) -> Option<Pointer<V>>
    where
        T: TypeMapKey<Value = Pointer<V>> + Send + Sync + 'static,
        V: Send + Sync + 'static,
    {
        let map_read = self.0.read().await;
        let map = map_read.get(&guild_id)?;
        map.read().await.get::<T>().cloned()
    }

    pub async fn get_cache<T, K, V>(&self, guild_id: GuildId) -> Option<T::Value>
    where
        T: TypeMapKey<Value = Pointer<Cache<K, V>>> + Send + Sync + 'static,
        K: Hash + Eq + Send + Sync + 'static,
        V: Send + Sync + 'static,
    {
        let map_read = self.0.read().await;
        let map = map_read.get(&guild_id)?;
        map.read().await.get::<T>().cloned()
    }

    pub async fn insert<T, V>(&self, guild_id: GuildId, data: V) -> Result<Pointer<V>, String>
    where
        T: TypeMapKey<Value = Pointer<V>> + Send + Sync + 'static,
        V: Send + Sync + 'static,
    {
        // Either Update existing or insert new
        let guilds = {
            let ptr = self.0.read().await;
            ptr.get(&guild_id).cloned()
        };
        if let Some(guild_map) = guilds {
            let ptr = Pointer::new(data);
            guild_map.write().await.insert::<T>(ptr.clone());
            return Ok(ptr);
        }
        Err(format!(
            "Tried to insert data into non-existing guild data map: {}",
            guild_id
        ))
    }

    pub async fn insert_ptr<T>(&self, guild_id: GuildId, data: T) -> Result<Pointer<T>, String>
    where
        T: Send + Sync + 'static,
    {
        // Either Update existing or insert new
        let guilds = {
            let ptr = self.0.read().await;
            ptr.get(&guild_id).cloned()
        };
        if let Some(guild_map) = guilds {
            let ptr = Pointer::new(data);
            guild_map.write().await.insert::<Pointer<T>>(ptr.clone());
            return Ok(ptr);
        }
        Err(format!(
            "Tried to insert data into non-existing guild data map: {}",
            guild_id
        ))
    }

    pub async fn insert_cache<T, K, V>(
        &self,
        guild_id: GuildId,
        data: Cache<K, V>,
    ) -> Result<Pointer<Cache<K, V>>, String>
    where
        T: TypeMapKey<Value = Pointer<Cache<K, V>>> + Send + Sync + 'static,
        K: Hash + Eq + Send + Sync + 'static,
        V: Send + Sync + 'static,
    {
        // Either Update existing or insert new
        let guilds = {
            let ptr = self.0.read().await;
            ptr.get(&guild_id).cloned()
        };
        if let Some(guild_map) = guilds {
            let ptr = Pointer::new(data);
            guild_map.write().await.insert::<T>(ptr.clone());
            return Ok(ptr);
        }
        Err(format!(
            "Tried to insert data into non-existing guild data map: {}",
            guild_id
        ))
    }

    pub async fn get_cloned<T, V>(&self, guild_id: GuildId) -> Option<V>
    where
        T: TypeMapKey<Value = Pointer<V>> + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        let ptr = self.get::<T, V>(guild_id).await?;
        Some(ptr.make_clone().await)
    }

    pub async fn get_cloned_ptr<T>(&self, guild_id: GuildId) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        match self.get_ptr::<T>(guild_id).await {
            Some(ptr) => Some(ptr.make_clone().await),
            None => None,
        }
    }

    pub async fn remove<T>(&self, guild_id: GuildId)
    where
        T: Send + Sync + 'static,
    {
        let guilds = {
            let ptr = self.0.read().await;
            ptr.get(&guild_id).cloned()
        };
        if let Some(guild_map) = guilds {
            guild_map.write().await.remove::<Pointer<T>>();
            return;
        }
        error!(
            "Tried to remove data from non-existing guild data map: {}",
            guild_id
        );
    }

    pub async fn map(&self, guild_id: GuildId) -> Option<GuildMap> {
        let guilds = {
            let ptr = self.0.read().await;
            ptr.get(&guild_id).cloned()
        };
        if let Some(guild_map) = guilds {
            return Some(guild_map);
        }
        error!("Tried to access non-existing guild data map: {}", guild_id);
        None
    }
}

impl TypeMapKey for Guilds {
    type Value = Pointer<HashMap<GuildId, GuildMap>>;
}

#[async_trait]
impl ContextExtractor for Guilds {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        let shard_data = ShardData::get(ctx).await?;
        Some(shard_data.guilds)
    }
}

#[async_trait]
impl<T> Extractor<T> for Guilds
where
    T: Send + Sync + Sized + 'static,
{
    async fn extract(ctx: &Context, _: &T, _: &Pointer<Parser>) -> Option<Self> {
        Guilds::extract_context(ctx).await
    }
}

#[async_trait]
impl<T> ContextEventExtractor<T> for Pointer<PartialGuild>
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, action: &T) -> Option<Self> {
        let guild_id = GuildId::extract_event(action).await?;
        let guilds = Guilds::extract_context(ctx).await?;
        guilds.get_ptr::<PartialGuild>(guild_id).await
    }
}

#[async_trait]
impl<CommandAction> Extractor<CommandAction> for Pointer<PartialGuild>
where
    CommandAction: Send + Sync + 'static,
    GuildId: EventExtractor<CommandAction>,
{
    async fn extract(ctx: &Context, action: &CommandAction, _: &Pointer<Parser>) -> Option<Self> {
        Pointer::<PartialGuild>::extract_context_event(ctx, action).await
    }
}

#[async_trait]
impl Extractor<CommandAction> for PartialGuild {
    async fn extract(ctx: &Context, action: &CommandAction, p: &Pointer<Parser>) -> Option<Self> {
        let ptr = Pointer::<PartialGuild>::extract(ctx, action, p).await?;
        Some(ptr.make_clone().await)
    }
}

#[async_trait]
impl Extractor<Event> for PartialGuild {
    async fn extract(ctx: &Context, ev: &Event, p: &Pointer<Parser>) -> Option<Self> {
        match ev {
            Event::GuildUpdate(guild_update) => Some(guild_update.guild.clone()),
            Event::GuildDelete(guild_delete) => {
                let guild_id = guild_delete.guild.id;
                let guilds = Guilds::extract(ctx, ev, p).await?;
                let guild = guilds.get_ptr(guild_id).await?;
                Some(guild.make_clone().await)
            }
            _ => {
                let ptr = Pointer::<PartialGuild>::extract_context_event(ctx, ev).await?;
                Some(ptr.make_clone().await)
            }
        }
    }
}

#[async_trait]
impl<T> ContextEventExtractor<T> for GuildMap
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self> {
        let guild_id = GuildId::extract_event(ev).await?;
        let guilds = Guilds::extract_context(ctx).await?;
        guilds.map(guild_id).await
    }
}

#[async_trait]
impl<T> Extractor<T> for GuildMap
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract(ctx: &Context, ev: &T, _: &Pointer<Parser>) -> Option<Self> {
        GuildMap::extract_context_event(ctx, ev).await
    }
}

#[async_trait]
pub trait HTTPGetter<Key, T> {
    async fn fetch(&self, http: &Http, key: Key) -> Option<T>;
}
