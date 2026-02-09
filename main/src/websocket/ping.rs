use framework::{ShardData, extractors::ShardManagerContainer, websocket::WebSocketWriter};
use serde::{Deserialize, Serialize};
use serenity::all::ConnectionStage;
use utils::{DataType, HttpType, info};

use crate::{data::member_list::MemberList, websocket::SocketSendEvent};

#[derive(Deserialize, Serialize, Debug)]
pub struct PingPayload {
    pub timestamp: i64,
    pub avg_ping: Option<i64>,
}

impl PingPayload {
    pub fn new(avg_ping: Option<i64>) -> Self {
        PingPayload {
            timestamp: chrono::Utc::now().timestamp_millis(),
            avg_ping,
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

#[derive(Serialize)]
pub struct ShardPingPayload {
    pub ping: PingPayload,
    pub shards: Vec<ShardUpdatePayload>,
}

pub async fn handle(payload: PingPayload, data: DataType, _: HttpType) {
    if let Some(writer) = WebSocketWriter::get(&data).await {
        let elapsed = chrono::Utc::now().timestamp_millis() - payload.timestamp;
        let avg_ping = payload.avg_ping.map(|p| (p + elapsed) / 2);

        let response = ShardPingPayload {
            ping: PingPayload::new(avg_ping.or(Some(elapsed))),
            shards: get_shards(&data).await,
        };

        if let Err(e) = (writer.send(SocketSendEvent::BotUpdate, response)).await {
            info!("Error sending pong: {}", e);
        }
    }
}

pub async fn get_shards(data: &DataType) -> Vec<ShardUpdatePayload> {
    let mut payloads = vec![];
    if let Some(ShardManagerContainer(shard_manager)) = ShardManagerContainer::get(data).await {
        let shards = shard_manager.shards_instantiated().await;
        for shard_id in shards {
            if let Some(runner) = shard_manager.runners.lock().await.get(&shard_id)
                && let Some(shard_data) = ShardData::get(shard_id, data).await
            {
                let member_count = {
                    let mut count: u32 = 0;
                    let guilds_ptr = shard_data.guilds.ptr();
                    for (_id, map) in guilds_ptr.read().await.iter() {
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
                payloads.push(payload);
            }
        }
    }
    payloads
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
