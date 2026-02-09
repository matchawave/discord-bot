use framework::websocket::{WebSocketProcessorBuilder, WebSocketWriter};
use serde::{Deserialize, Serialize};
use utils::{DataType, HttpType, error};

use crate::websocket::ping::PingPayload;

pub mod ping;

pub fn get_websocket_connection() -> WebSocketProcessorBuilder<SocketReceiveEvent> {
    WebSocketProcessorBuilder::<SocketReceiveEvent>::default()
        .add_callback(SocketReceiveEvent::Ready, handle_ready)
        .add_callback(SocketReceiveEvent::BotPong, ping::handle)
}

async fn handle_ready(_: (), data: DataType, _: HttpType) {
    if let Some(writer) = WebSocketWriter::get(&data).await {
        let ping = PingPayload {
            timestamp: chrono::Utc::now().timestamp_millis(),
            avg_ping: None,
        };
        let shards = ping::get_shards(&data).await;
        let response = ping::ShardPingPayload { ping, shards };

        if let Err(e) = writer.send(SocketSendEvent::BotUpdate, response).await {
            error!("Error sending initial ping: {}", e);
        }
    }
}

#[derive(Hash, Eq, PartialEq, Serialize)]
pub enum SocketSendEvent {
    BotUpdate,
}

#[derive(Hash, Eq, PartialEq, Deserialize, Default)]
pub enum SocketReceiveEvent {
    #[default]
    Ready,
    BotPong,
}
