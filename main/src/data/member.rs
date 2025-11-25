use std::collections::HashMap;

use framework::{DataExtract, DataExtractable, Extractable, extractors::Extractor};
use serenity::{
    all::{GuildId, Member, UserId},
    prelude::TypeMapKey,
};
use utils::{MemberData, Pointer};

type MemberDataMap = HashMap<UserId, MemberData>;

#[derive(Clone, Default, DataExtractable, DataExtract)]
pub struct MembersInfo(Pointer<HashMap<GuildId, Pointer<MemberDataMap>>>);

impl MembersInfo {
    pub async fn insert(&self, guild_id: GuildId, map: MemberDataMap) {
        let mut data = self.0.write().await;
        data.insert(guild_id, Pointer::new(map));
    }
    pub async fn get(&self, guild_id: &GuildId) -> Option<Pointer<MemberDataMap>> {
        let data = self.0.read().await;
        data.get(guild_id).cloned()
    }
    pub async fn remove(&self, guild_id: &GuildId) {
        let mut data = self.0.write().await;
        data.remove(guild_id);
    }

    pub async fn add(&self, member: &Member) {
        if let Some(guild_map) = self.0.read().await.get(&member.guild_id)
            && let Some(join_date) = member.joined_at
        {
            let mut guild_map = guild_map.write().await;
            guild_map.insert(
                member.user.id,
                MemberData {
                    is_bot: member.user.bot,
                    join_date,
                },
            );
        }
    }
    pub async fn subtract(&self, member: &Member) {
        if let Some(guild_map) = self.0.read().await.get(&member.guild_id) {
            let mut guild_map = guild_map.write().await;
            guild_map.remove(&member.user.id);
        }
    }

    pub async fn debug_print(&self) -> String {
        let data = self.0.read().await;
        if data.is_empty() {
            return "No guilds in MembersInfo".into();
        }
        let mut output = String::new();
        output.push_str("\n--- Members Info Summary ---");
        for (guild_id, members_map_ptr) in data.iter() {
            output.push_str(&format!("\nGuild ID: {}", guild_id));
            let members_map = members_map_ptr.read().await;
            output.push_str(&format!("\nTotal Members: {}", members_map.len()));
            for (user_id, member_data) in members_map.iter() {
                output.push_str(&format!(
                    "\n    {}, Bot: {}, Joined: {}",
                    user_id, member_data.is_bot, member_data.join_date
                ));
            }
        }
        output.push_str("\n----------------------------");
        output
    }
}

impl TypeMapKey for MembersInfo {
    type Value = Pointer<HashMap<GuildId, Pointer<MemberDataMap>>>;
}
