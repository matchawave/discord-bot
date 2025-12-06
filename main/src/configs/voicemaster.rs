use super::VoiceConfig;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serenity::all::{ChannelId, UserId};
use std::collections::HashMap;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoiceMasterConfig {
    #[serde_as(as = "Vec<(_, _)>")]
    masters: HashMap<ChannelId, (Option<ChannelId>, Option<u64>)>,
    #[serde(skip)] // Active channels are not serialized, they are runtime only
    actives: HashMap<ChannelId, UserId>,
    #[serde_as(as = "Vec<(_, _)>")]
    config: HashMap<ChannelId, VoiceConfig>,
}

impl VoiceMasterConfig {
    pub fn new(
        masters: HashMap<ChannelId, (Option<ChannelId>, Option<u64>)>,
        config: HashMap<ChannelId, VoiceConfig>,
    ) -> Self {
        Self {
            masters,
            config,
            actives: HashMap::new(),
        }
    }

    /// Returns the parent id of the master channel if it exists
    pub fn is_master(&self, channel: ChannelId) -> Option<(Option<ChannelId>, Option<u64>)> {
        self.masters.get(&channel).copied()
    }

    /// Insert a voice channel that was created by a user
    pub fn insert_active(&mut self, channel: ChannelId, owner: UserId) {
        self.actives.insert(channel, owner);
    }

    /// Remove a voice channel that was created by a user
    pub fn remove_active(&mut self, channel: ChannelId) -> Option<(ChannelId, UserId)> {
        self.actives.remove(&channel).map(|owner| (channel, owner))
    }

    /// Get the owner of an active voice channel
    pub fn get_active(&self, channel: ChannelId) -> Option<(ChannelId, UserId)> {
        self.actives.get(&channel).map(|owner| (channel, *owner))
    }

    /// Set the configuration for a master channel (If needed)
    pub fn insert_config(&mut self, channel: ChannelId, config: VoiceConfig) {
        self.config.insert(channel, config);
    }

    /// Get the configuration for a master channel (If it exists)
    pub fn get_config(&self, channel: ChannelId) -> Option<&VoiceConfig> {
        self.config.get(&channel)
    }
}
