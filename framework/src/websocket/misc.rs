use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct WsEnvelope<T> {
    pub(super) event: T,
    pub(super) data: Value,
}

impl<T> WsEnvelope<T> {
    pub(super) fn new<U>(event: T, data: U) -> Self
    where
        U: Serialize,
    {
        let data = serde_json::to_value(data).unwrap_or(Value::Null);
        Self { event, data }
    }
}

#[derive(Hash, Eq, PartialEq, Serialize)]
pub enum SocketSendEvent {
    Ping,
}

#[derive(Hash, Eq, PartialEq, Deserialize)]
pub enum SocketReceiveEvent {
    Pong,
}

pub enum SocketEventPayload {
    Pong,
}
