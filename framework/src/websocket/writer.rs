use std::sync::Arc;

use crate::extractors::ContextExtractor;

use super::WebSocketStreamType;
use colored::Colorize;
use serde::Serialize;
use serenity::{
    all::Context,
    async_trait,
    futures::{SinkExt, stream::SplitSink},
    prelude::TypeMapKey,
};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use utils::error;

pub(super) type SocketWriter = SplitSink<WebSocketStreamType, Message>;

#[derive(Clone)]
pub struct WebSocketWriter(Arc<Mutex<SocketWriter>>);
impl WebSocketWriter {
    pub fn new(writer: SocketWriter) -> Self {
        Self(Arc::new(Mutex::new(writer)))
    }

    pub async fn send<T>(&self, message: T)
    where
        T: Serialize,
    {
        let ws_text = "websocket".yellow();
        let msg = match serde_json::to_string(&message) {
            Ok(s) => Message::Text(Utf8Bytes::from(s)),
            Err(e) => {
                error!("({ws_text}) Error serializing message: {:?}", e);
                return;
            }
        };

        if let Err(e) = self.0.lock().await.send(msg).await {
            error!("({ws_text}) Error sending message: {:?}", e);
        }
    }

    pub async fn flush(&self) {
        if let Err(e) = self.0.lock().await.flush().await {
            let ws_text = "websocket".yellow();
            error!("({ws_text}) Error flushing message: {:?}", e);
        }
    }
}

impl TypeMapKey for WebSocketWriter {
    type Value = Arc<Mutex<SocketWriter>>;
}

#[async_trait]
impl ContextExtractor for WebSocketWriter {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        let data_read = ctx.data.read().await;
        data_read.get::<WebSocketWriter>().cloned().map(Self)
    }
}
