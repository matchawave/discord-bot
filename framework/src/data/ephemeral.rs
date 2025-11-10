use std::collections::HashMap;

use serenity::{
    all::{ChannelId, GuildId, Message, MessageId},
    async_trait,
};
use utils::{Pointer, error, info};

use crate::{build_process, processes::ProcessLoop};

build_process!(Ephemerals, Pointer<HashMap<Ephemeral, std::time::Instant>>);

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Ephemeral {
    channel_id: ChannelId,
    message_id: MessageId,
}

impl Ephemeral {
    pub fn new(msg: &Message) -> Self {
        let message_id = msg.id;
        let channel_id = msg.channel_id;

        Self {
            channel_id,
            message_id,
        }
    }
}

#[async_trait]
impl ProcessLoop for Ephemerals {
    async fn process(&self, http: std::sync::Arc<serenity::http::Http>) {
        let map = self.0.make_clone().await;
        let now = std::time::Instant::now();
        for (key, &time) in map.iter() {
            if now > time {
                info!(
                    "(ephemerals) Deleting ephemeral message {} in channel {}",
                    key.message_id, key.channel_id
                );
                if let Err(e) = http
                    .delete_message(
                        key.channel_id,
                        key.message_id,
                        Some("Ephemeral message cleanup"),
                    )
                    .await
                {
                    error!(
                        "(ephemerals) Failed to delete ephemeral message {} in channel {}: {}",
                        key.message_id, key.channel_id, e
                    );
                }
                let mut map = self.0.write().await;
                map.remove(key);
            }
        }
    }
}
