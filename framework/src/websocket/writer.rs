use std::sync::Arc;

use crate::extractors::{ContextExtractor, Extractor};

use super::{WebSocketStreamType, misc::WsEnvelope};
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
use utils::{Parser, Pointer, error};

pub(super) type SocketWriter = SplitSink<WebSocketStreamType, Message>;

#[derive(Clone)]
pub struct WebSocketWriter(Arc<Mutex<SocketWriter>>);
impl WebSocketWriter {
    pub fn new(writer: SocketWriter) -> Self {
        Self(Arc::new(Mutex::new(writer)))
    }

    pub async fn send<T, U>(&self, event: T, data: U) -> Result<(), String>
    where
        T: Serialize + std::hash::Hash + Eq,
        U: Serialize,
    {
        let message = WsEnvelope::new(event, data);
        let msg: Message = match serde_json::to_string(&message) {
            Ok(s) => Message::Text(Utf8Bytes::from(s)),
            Err(e) => return Err(format!("Error serializing message: {:?}", e)),
        };

        if let Err(e) = self.0.lock().await.send(msg).await {
            return Err(format!("Error sending message: {:?}", e));
        }
        Ok(())
    }

    pub async fn flush(&self) {
        if let Err(e) = self.0.lock().await.flush().await {
            let ws_text = "websocket".yellow();
            error!("({ws_text}) Error flushing message: {:?}", e);
        }
    }

    pub async fn get(data: &utils::DataType) -> Option<Self> {
        let data_read = data.read().await;
        data_read.get::<WebSocketWriter>().cloned().map(Self)
    }
}

impl TypeMapKey for WebSocketWriter {
    type Value = Arc<Mutex<SocketWriter>>;
}

#[async_trait]
impl ContextExtractor for WebSocketWriter {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        Self::get(&ctx.data).await
    }
}

#[async_trait]
impl<T> Extractor<T> for WebSocketWriter
where
    T: Send + Sync + 'static,
{
    async fn extract(ctx: &Context, _: &T, _: &Pointer<Parser>) -> Option<Self> {
        Self::extract_context(ctx).await
    }
}
