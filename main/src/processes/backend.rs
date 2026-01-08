use chrono::DateTime;
use framework::{
    build_process, extractors::ShardManagerContainer, processes::ProcessLoop,
    websocket::WebSocketWriter,
};
use serde::Serialize;
use serenity::{all::ConnectionStage, async_trait};

use crate::{global::shard_list::ShardList, websocket::SocketSendEvent};

build_process!(ShardUpdater, DateTime<chrono::Utc>);

#[async_trait]
impl ProcessLoop for ShardUpdater {
    async fn process(&self, _: utils::HttpType, data: utils::DataType) {
        loop {
            if let Some(socket_writer) = WebSocketWriter::get(&data).await
                && let Some(ShardManagerContainer(shard_manager)) =
                    ShardManagerContainer::get(&data).await
                && let Some(shard_list) = ShardList::from_data(&data).await
            {
                let shards = shard_manager.shards_instantiated().await;
                for shard_id in shards {
                    if let Some(runner) = shard_manager.runners.lock().await.get(&shard_id)
                        && let Some(shard_data) = shard_list.get_ptr(shard_id).await
                    {
                        let shard_data = shard_data.read().await;
                        let payload = ShardUpdatePayload {
                            shard_id: shard_id.get(),
                            status: connection_status(runner.stage),
                            latency_ms: runner.latency.map(|d| d.as_millis()),
                            servers: shard_data.servers.iter().map(|id| id.to_string()).collect(),
                            members: shard_data.members,
                        };

                        if let Err(e) = socket_writer
                            .send(SocketSendEvent::ShardUpdate, payload)
                            .await
                        {
                            utils::error!(
                                "Error sending shard update for shard {}: {}",
                                shard_id.get(),
                                e
                            );
                        }
                    }
                }
            };
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }
}

#[derive(Serialize)]
pub struct ShardUpdatePayload {
    pub shard_id: u32,
    pub status: String,
    pub latency_ms: Option<u128>,
    pub servers: Vec<String>,
    pub members: u32,
}

fn connection_status(stage: ConnectionStage) -> String {
    match stage {
        ConnectionStage::Connected => "connected",
        ConnectionStage::Connecting => "connecting",
        ConnectionStage::Disconnected => "disconnected",
        ConnectionStage::Handshake => "handshake",
        ConnectionStage::Identifying => "identifying",
        ConnectionStage::Resuming => "resuming",
        _ => "unknown",
    }
    .to_string()
}
