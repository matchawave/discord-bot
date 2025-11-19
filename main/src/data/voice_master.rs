use std::collections::HashMap;

use framework::{DataExtractable, DefaultExtract, extractors::Extractor};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serenity::{
    all::{ChannelId, GuildId, UserId},
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};
use utils::Pointer;

use crate::global::VoiceConfig;

#[derive(Clone, Default, DefaultExtract)]
pub struct VoiceMasters(Pointer<HashMap<GuildId, Pointer<VoiceMasterConfig>>>);

impl VoiceMasters {
    pub async fn insert(&self, guild_id: GuildId, config: VoiceMasterConfig) {
        let mut data = self.0.write().await;
        data.insert(guild_id, Pointer::new(config));
    }

    pub async fn get_cloned(&self, guild_id: GuildId) -> Option<VoiceMasterConfig> {
        let data = self.0.read().await;
        match data.get(&guild_id).cloned() {
            Some(config) => Some(config.make_clone().await),
            None => None,
        }
    }

    pub async fn get(&self, guild_id: &GuildId) -> Option<Pointer<VoiceMasterConfig>> {
        let data = self.0.read().await;
        data.get(guild_id).cloned()
    }

    pub async fn remove(&self, guild_id: GuildId) {
        let mut data = self.0.write().await;
        data.remove(&guild_id);
    }
}

impl TypeMapKey for VoiceMasters {
    type Value = Pointer<HashMap<GuildId, Pointer<VoiceMasterConfig>>>;
}

impl DataExtractable for VoiceMasters {
    fn init(map: &mut TypeMap) {
        let mut data = HashMap::new();
        let mut config = VoiceMasterConfig::new(vec![MasterVoiceChannel::new(
            ChannelId::from(851183230359306251),
            None,
        )]);
        config.set_config(VoiceConfig {
            name: Some("🔊 {user.display_name}'s Channel".to_string()),
            bitrate: None,
            user_limit: Some(5),
            locked: None,
        });
        data.insert(GuildId::from(851102546470371338), Pointer::new(config));

        map.insert::<VoiceMasters>(Pointer::new(data));
    }

    fn retrieve(map: &std::sync::Arc<TypeMap>) -> Option<Self>
    where
        Self: Sized,
    {
        map.get::<VoiceMasters>().cloned().map(VoiceMasters)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoiceMasterConfig {
    masters: Vec<MasterVoiceChannel>,
    actives: Vec<ActiveVoiceChannel>,
    config: Option<VoiceConfig>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveVoiceChannel {
    pub id: ChannelId,
    pub owner: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterVoiceChannel {
    pub id: ChannelId,
    pub category: Option<ChannelId>,
}

impl MasterVoiceChannel {
    pub fn new(id: ChannelId, category: Option<ChannelId>) -> Self {
        Self { id, category }
    }
}

impl VoiceMasterConfig {
    pub fn new(masters: Vec<MasterVoiceChannel>) -> Self {
        Self {
            masters,
            actives: Vec::new(),
            config: None,
        }
    }

    pub fn is_master(&self, channel: ChannelId) -> Option<MasterVoiceChannel> {
        self.masters
            .par_iter()
            .find_first(|c| c.id == channel)
            .cloned()
    }

    pub fn add_active_channel(&mut self, channel: ChannelId, owner: UserId) -> ActiveVoiceChannel {
        let active_channel = ActiveVoiceChannel { id: channel, owner };
        self.actives.push(active_channel.clone());
        active_channel
    }

    pub fn remove_active_channel(&mut self, channel: ChannelId) -> Option<ActiveVoiceChannel> {
        match self.actives.par_iter().position_first(|c| c.id == channel) {
            Some(pos) => Some(self.actives.remove(pos)),
            None => None,
        }
    }

    pub fn is_active(&self, channel: ChannelId) -> bool {
        self.actives.par_iter().any(|c| c.id == channel)
    }

    pub fn is_owner(&self, channel: ChannelId, user: UserId) -> bool {
        self.actives
            .par_iter()
            .any(|c| c.id == channel && c.owner == user)
    }

    pub fn set_config(&mut self, config: VoiceConfig) {
        self.config = Some(config);
    }

    pub fn config(&self) -> Option<&VoiceConfig> {
        self.config.as_ref()
    }
}
