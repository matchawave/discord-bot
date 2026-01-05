use framework::websocket::WebSocketWriter;
use serde::{Deserialize, Serialize};
use utils::{DataType, HttpType, info};

use crate::websocket::SocketSendEvent;

#[derive(Deserialize, Serialize, Debug)]
pub struct PingPayload {
    pub timestamp: i64,
    pub previous_timestamp: Option<i64>,
}

pub async fn handle(payload: PingPayload, data: DataType, _: HttpType) {
    let elapsed = chrono::Utc::now().timestamp_millis() - payload.timestamp;
    if let Some(writer) = WebSocketWriter::get(&data).await {
        let pong_payload = PingPayload {
            timestamp: chrono::Utc::now().timestamp_millis(),
            previous_timestamp: Some(elapsed),
        };
        if let Err(e) = writer.send(SocketSendEvent::Ping, pong_payload).await {
            info!("Error sending pong: {}", e);
        }
    }
}
