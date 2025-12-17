use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, PermissionOverwrite, PermissionOverwriteType, Permissions, UserId};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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

impl VoiceConfig {
    pub fn permissions(&self, guild_id: GuildId) -> Option<[PermissionOverwrite; 2]> {
        let user_id = self.locked?;

        let whitelist = PermissionOverwrite {
            allow: Permissions::CONNECT,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member(user_id),
        };
        let blacklist = PermissionOverwrite {
            allow: Permissions::empty(),
            deny: Permissions::CONNECT,
            kind: PermissionOverwriteType::Role(guild_id.everyone_role()),
        };
        Some([whitelist, blacklist])
    }
}
