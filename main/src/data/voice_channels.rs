use std::collections::HashMap;

use framework::{DataExtract, DataExtractable, Extractable, extractors::Extractor};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serenity::{
    all::{ChannelId, GuildId, UserId},
    prelude::TypeMapKey,
};
use utils::{Pointer, debug, info};

type VoiceChannelMembers = Vec<UserId>;

#[derive(Clone, DataExtractable, DataExtract)]
pub struct ChannelMembers(Pointer<HashMap<(GuildId, ChannelId), Pointer<VoiceChannelMembers>>>);

impl TypeMapKey for ChannelMembers {
    type Value = Pointer<HashMap<(GuildId, ChannelId), Pointer<Vec<UserId>>>>;
}

impl ChannelMembers {
    pub async fn insert(&self, guild_id: GuildId, channel_id: ChannelId, user_id: UserId) {
        let key = (guild_id, channel_id);
        let mut data = self.0.write().await;
        let entry = data.entry(key).or_insert_with(|| Pointer::new(Vec::new()));
        let mut members = entry.write().await;
        if !members.contains(&user_id) {
            debug!(
                "Inserting user {} into voice channel {} in guild {}",
                user_id, channel_id, guild_id
            );
            members.push(user_id);
        }
    }

    pub async fn insert_multiple(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
        user_ids: Vec<UserId>,
    ) {
        let key = (guild_id, channel_id);
        let mut data = self.0.write().await;
        let entry = data.entry(key).or_insert_with(|| Pointer::new(Vec::new()));
        let mut members = entry.write().await;
        for user_id in user_ids {
            if !members.contains(&user_id) {
                members.push(user_id);
            }
        }
    }

    pub async fn remove(&self, guild_id: GuildId, channel_id: ChannelId, user_id: UserId) {
        let key = (guild_id, channel_id);
        let entry = { (self.0.read().await).get(&key).cloned() };
        if let Some(entry) = &entry {
            let index = {
                let members = entry.read().await;
                members.par_iter().position_first(|&id| id == user_id)
            };
            if let Some(idx) = index {
                let mut members = entry.write().await;
                members.remove(idx);
                debug!(
                    "Removed user {} from voice channel {} in guild {}",
                    user_id, channel_id, guild_id
                );
            }
        }
    }

    pub async fn remove_user(&self, user_id: UserId) {
        for ((g_id, c_id), entry) in (self.0.read().await).iter() {
            let index = {
                let members = entry.read().await;
                members.par_iter().position_first(|id| *id == user_id)
            };
            if let Some(idx) = index {
                let mut members = entry.write().await;
                members.remove(idx);
                info!(
                    "Removed user {} from voice channel {} in guild {}",
                    user_id, c_id, g_id
                );
                break;
            }
        }
    }

    pub async fn remove_channel(&self, guild_id: GuildId, channel_id: ChannelId) {
        let key = (guild_id, channel_id);
        self.0.write().await.remove(&key);
    }

    pub async fn clear_guild(&self, guild_id: GuildId) {
        let filtered = {
            (self.0.read().await)
                .par_iter()
                .filter_map(|(&(g_id, c_id), _)| {
                    if g_id == guild_id {
                        Some((g_id, c_id))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        let mut data = self.0.write().await;
        for key in filtered {
            data.remove(&key);
        }
    }

    pub async fn get(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Option<Pointer<Vec<UserId>>> {
        let key = (guild_id, channel_id);
        self.0.read().await.get(&key).cloned()
    }

    pub async fn get_cloned(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Option<Vec<UserId>> {
        let key = (guild_id, channel_id);
        match (self.0.read().await).get(&key) {
            Some(entry) => Some(entry.make_clone().await),
            None => None,
        }
    }
}
