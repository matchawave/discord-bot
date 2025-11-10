use dashmap::DashMap;
use framework::data::DataExt;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{ChannelId, GuildId, UserId},
    prelude::{TypeMap, TypeMapKey},
};
use utils::Pointer;

pub struct VoiceMasters(Pointer<DashMap<GuildId, Pointer<VoiceMasterConfig>>>);

impl VoiceMasters {}

impl DataExt for VoiceMasters {
    fn init(map: &mut TypeMap) {
        map.insert::<Self>(Pointer::new(DashMap::new()));
    }

    fn retrieve(map: &std::sync::Arc<TypeMap>) -> Self {
        map.get::<Self>()
            .cloned()
            .map(Self)
            .expect("VoiceMasters data not initialized")
    }
}

impl TypeMapKey for VoiceMasters {
    type Value = Pointer<DashMap<GuildId, Pointer<VoiceMasterConfig>>>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoiceMasterConfig {
    master: Vec<MasterVoiceChannel>,
    active: Vec<ActiveVoiceChannel>,
    config: Option<VoiceConfig>,
    parent_id: Option<ChannelId>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveVoiceChannel {
    pub id: ChannelId,
    pub owner: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterVoiceChannel {
    pub id: ChannelId,
    pub category: Option<u64>,
}

impl MasterVoiceChannel {
    pub fn new(id: ChannelId, category: Option<u64>) -> Self {
        Self { id, category }
    }
}

impl VoiceMasterConfig {
    pub fn new(master: Vec<MasterVoiceChannel>) -> Self {
        Self {
            master,
            active: Vec::new(),
            config: None,
            parent_id: None,
        }
    }

    pub fn is_master(&self, channel: &ChannelId) -> bool {
        self.master.iter().any(|c| c.id == *channel)
    }

    pub fn add_active_channel(&mut self, channel: ChannelId, owner: UserId) -> ActiveVoiceChannel {
        let active_channel = ActiveVoiceChannel { id: channel, owner };
        self.active.push(active_channel.clone());
        active_channel
    }

    pub fn remove_active_channel(&mut self, channel: &ChannelId) -> Option<ActiveVoiceChannel> {
        match self.active.iter().position(|c| c.id == *channel) {
            Some(pos) => Some(self.active.remove(pos)),
            None => None,
        }
    }

    pub fn is_active(&self, channel: &ChannelId) -> bool {
        self.active.iter().any(|c| c.id == *channel)
    }
    pub fn is_owner(&self, channel: &ChannelId, user: &UserId) -> bool {
        self.active
            .iter()
            .any(|c| c.id == *channel && c.owner == *user)
    }

    pub fn set_config(&mut self, config: VoiceConfig) {
        self.config = Some(config);
    }

    pub fn set_parent_id(&mut self, parent_id: ChannelId) {
        self.parent_id = Some(parent_id);
    }

    pub fn config(&self) -> Option<&VoiceConfig> {
        self.config.as_ref()
    }

    pub fn parent_id(&self) -> Option<&ChannelId> {
        self.parent_id.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<UserId>,
}
