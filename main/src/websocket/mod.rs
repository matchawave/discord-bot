use framework::websocket::{WebSocketProcessorBuilder, WebSocketWriter};
use serde::{Deserialize, Serialize};
use utils::{DataType, HttpType, error};

use crate::websocket::ping::PingPayload;

pub mod ping;

pub fn get_websocket_connection() -> WebSocketProcessorBuilder<SocketReceiveEvent> {
    WebSocketProcessorBuilder::<SocketReceiveEvent>::default()
        .add_callback(SocketReceiveEvent::Ready, handle_ready)
        .add_callback(SocketReceiveEvent::Pong, ping::handle)
}

async fn handle_ready(_: (), data: DataType, _: HttpType) {
    let ping = PingPayload {
        timestamp: chrono::Utc::now().timestamp_millis(),
        previous_timestamp: None,
    };
    if let Some(writer) = WebSocketWriter::get(&data).await
        && let Err(e) = writer.send(SocketSendEvent::Ping, ping).await
    {
        error!("Error sending initial ping: {}", e);
    }
}

#[derive(Hash, Eq, PartialEq, Serialize)]
pub enum SocketSendEvent {
    Ping,
}

#[derive(Hash, Eq, PartialEq, Deserialize, Default)]
pub enum SocketReceiveEvent {
    #[default]
    Ready,
    Pong,
}
