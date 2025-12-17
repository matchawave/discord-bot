use super::VoiceConfig;
use framework::{
    ShardData,
    extractors::{ContextEventExtractor, EventExtractor},
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serenity::{
    all::{ChannelId, Context, GuildId, UserId},
    async_trait,
    prelude::TypeMapKey,
};
use std::{collections::HashMap, time::Duration};
use utils::{Parser, Pointer};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoiceMasterConfig {
    #[serde_as(as = "Vec<(_, _)>")]
    pub masters: HashMap<ChannelId, (Option<ChannelId>, Option<Duration>)>,
    #[serde_as(as = "Vec<(_, _)>")]
    pub configs: HashMap<ChannelId, VoiceConfig>,
}

impl VoiceMasterConfig {
    pub fn new(
        masters: HashMap<ChannelId, (Option<ChannelId>, Option<Duration>)>,
        config: HashMap<ChannelId, VoiceConfig>,
    ) -> Self {
        Self {
            masters,
            configs: config,
        }
    }
}
