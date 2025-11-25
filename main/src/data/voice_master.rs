use std::collections::HashMap;

use framework::{DataExtract, Extractable, extractors::Extractor};

use serenity::{
    all::{ChannelId, GuildId},
    prelude::{TypeMap, TypeMapKey},
};
use utils::Pointer;

use crate::configs::{VoiceConfig, VoiceMasterConfig};

#[derive(Clone, Default, DataExtract)]
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

impl Extractable for VoiceMasters {
    fn init(map: &mut TypeMap) {
        let mut data = HashMap::new();
        let channel_id = ChannelId::from(851183230359306251);
        let voice_config = VoiceConfig {
            name: Some("🔊 {user.display_name}'s Channel".into()),
            bitrate: None,
            user_limit: Some(5),
            locked: None,
        };
        let mut master_config = HashMap::new();
        master_config.insert(channel_id, None);

        let mut config = VoiceMasterConfig::new(master_config, HashMap::new());
        config.insert_config(channel_id, voice_config);
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
