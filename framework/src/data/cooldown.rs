use std::collections::HashMap;

use serenity::{
    all::{GuildId, UserId},
    async_trait,
};
use utils::{Pointer, info};

use crate::{build_process, processes::ProcessLoop};

build_process!(Cooldowns, Pointer<HashMap<Cooldown, std::time::Instant>>);

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Cooldown {
    user_id: UserId,
    guild_id: GuildId,
    command_name: String,
}

impl Cooldown {
    pub fn new<T: Into<String>>(user_id: UserId, guild_id: GuildId, command_name: T) -> Self {
        Self {
            user_id,
            guild_id,
            command_name: command_name.into(),
        }
    }
}

#[async_trait]
impl ProcessLoop for Cooldowns {
    async fn process(&self, http: std::sync::Arc<serenity::http::Http>) {
        let map = self.0.make_clone().await;
        let now = std::time::Instant::now();
        for (key, &time) in map.iter() {
            if now > time {
                info!(
                    "(cooldowns) Removing expired cooldown for user {} in guild {} for command {}",
                    key.user_id, key.guild_id, key.command_name
                );
                let mut map = self.0.write().await;
                map.remove(key);
            }
        }
    }
}
