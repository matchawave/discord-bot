use framework::websocket::WebSocketWriter;
use serde::{Deserialize, Serialize};
use utils::{DataType, HttpType, info};

use crate::websocket::SocketSendEvent;

#[derive(Deserialize, Serialize, Debug)]
pub struct PingPayload {
    pub timestamp: i64,
    pub avg_ping: Option<i64>,
}

pub async fn handle(payload: PingPayload, data: DataType, _: HttpType) {
    let elapsed = chrono::Utc::now().timestamp_millis() - payload.timestamp;
    if let Some(writer) = WebSocketWriter::get(&data).await {
        let avg_ping = payload.avg_ping.map(|p| (p + elapsed) / 2);
        let pong_payload = PingPayload {
            timestamp: chrono::Utc::now().timestamp_millis(),
            avg_ping: avg_ping.or(Some(elapsed)),
        };
        if let Err(e) = writer.send(SocketSendEvent::BotPing, pong_payload).await {
            info!("Error sending pong: {}", e);
        }
    }
}
