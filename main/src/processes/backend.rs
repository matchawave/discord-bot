use chrono::DateTime;
use framework::{processes::ProcessLoop, websocket::WebSocketWriter};
use serenity::async_trait;
use tokio::sync::RwLock;

#[derive(Default)]
/// A process to track the last time a ping was received over the websocket connection to the backend.
/// This is useful for monitoring the health of the connection.
pub struct WebsocketPingProcess(pub RwLock<DateTime<chrono::Utc>>);

#[async_trait]
impl ProcessLoop for WebsocketPingProcess {
    async fn process(&self, _: utils::HttpType, data: utils::DataType) {}
}
