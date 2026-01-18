use chrono::DateTime;
use framework::{
    ShardData, build_process, extractors::ShardManagerContainer, processes::ProcessLoop,
    websocket::WebSocketWriter,
};
use serde::Serialize;
use serenity::{all::ConnectionStage, async_trait};
use utils::warning;

use crate::{data::member_list::MemberList, websocket::SocketSendEvent};

build_process!(ShardUpdater, DateTime<chrono::Utc>);

#[async_trait]
impl ProcessLoop for ShardUpdater {
    async fn process(&self, _: utils::HttpType, data: utils::DataType) {
        loop {
            if let Some(socket_writer) = WebSocketWriter::get(&data).await
                && let Some(ShardManagerContainer(shard_manager)) =
                    ShardManagerContainer::get(&data).await
            {
                let shards = shard_manager.shards_instantiated().await;
                for shard_id in shards {
                    if let Some(runner) = shard_manager.runners.lock().await.get(&shard_id)
                        && let Some(shard_data) = ShardData::get(shard_id, &data).await
                    {
                        let member_count = {
                            let mut count: u32 = 0;
                            let guilds_ptr = shard_data.guilds.ptr();
                            for (id, map) in guilds_ptr.read().await.iter() {
                                if let Some(list) = MemberList::from_map(map).await {
                                    count += list.len().await as u32;
                                    continue;
                                }
                            }
                            count
                        };

                        let payload = ShardUpdatePayload {
                            shard_id: shard_id.get(),
                            status: connection_status(runner.stage),
                            latency_ms: runner.latency.map(|d| d.as_millis()),
                            members: member_count,
                        };

                        if let Err(e) =
                            (socket_writer.send(SocketSendEvent::ShardUpdate, payload)).await
                        {
                            utils::error!("(Shard Updater) on shard {}: {}", shard_id.get(), e);
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
