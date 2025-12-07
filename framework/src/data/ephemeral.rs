use std::{collections::HashMap, sync::Arc};

use serenity::{
    all::{ChannelId, Context, Message, MessageId, ShardId},
    async_trait,
};
use utils::{Http, Pointer, error, info};

use crate::{
    build_process,
    extractors::{ContextExtractor, Extractor},
    processes::{ProcessLoop, ProcessManager},
};

build_process!(Ephemerals, HashMap<Ephemeral, std::time::Instant>);

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
    async fn process(&self, http: Http) {
        loop {
            let map = self.0.read().await.clone();
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
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

#[async_trait]
impl ContextExtractor for Arc<Ephemerals> {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        let p_manager = Arc::<ProcessManager>::extract_context(ctx).await?;
        p_manager.get::<Ephemerals>()
    }
}

#[async_trait]
impl<T> Extractor<T> for Arc<Ephemerals>
where
    T: Send + Sync + 'static,
{
    async fn extract(ctx: &Context, _: &T, _: &Pointer<utils::Parser>) -> Option<Self> {
        Arc::<Ephemerals>::extract_context(ctx).await
    }
}
