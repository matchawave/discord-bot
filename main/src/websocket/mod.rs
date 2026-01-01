use framework::websocket::{SocketReceiveEvent, WebSocketProcessorBuilder};
use serde_json::Value;
use utils::{DataType, HttpType};

pub fn get_websocket_connection() -> WebSocketProcessorBuilder {
    WebSocketProcessorBuilder::default().add_callback(SocketReceiveEvent::Pong, handle_ping)
}

async fn handle_ping(_: Option<Value>, _: DataType, _: HttpType) {
    // Handle ping event
}
