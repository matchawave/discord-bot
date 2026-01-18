use std::collections::HashMap;

use colored::Colorize;
use serde::de::DeserializeOwned;
use serenity::futures::{StreamExt, stream::SplitStream};
use tokio_tungstenite::tungstenite::Message;
use utils::{DataType, HttpType, error, info};

use super::{EventCallback, WebSocketStreamType, misc::WsEnvelope};

pub(super) type SocketReader = SplitStream<WebSocketStreamType>;

pub(super) async fn read_socket<T>(
    mut reader: SocketReader,
    callbacks: &HashMap<T, Vec<EventCallback>>,
    http: HttpType,
    data: DataType,
) where
    T: DeserializeOwned + Eq + std::hash::Hash + Default,
{
    let ws_text = "websocket".yellow();
    if let Some(callback_vec) = callbacks.get(&T::default()) {
        for callback in callback_vec.iter() {
            callback
                .call(serde_json::Value::Null, data.clone(), http.clone())
                .await;
        }
    }

    while let Some(message) = reader.next().await {
        match message {
            Ok(Message::Text(msg)) => {
                let envelope: WsEnvelope<T> = match serde_json::from_str(&msg) {
                    Ok(env) => env,
                    Err(e) => {
                        error!("({ws_text}) Error deserializing message: {:?}", e);
                        continue;
                    }
                };

                if let Some(callback_vec) = callbacks.get(&envelope.event) {
                    for callback in callback_vec.iter() {
                        callback
                            .call(envelope.data.clone(), data.clone(), http.clone())
                            .await;
                    }
                }
            }
            Ok(Message::Close(Some(close))) => {
                info!(
                    "({ws_text}) WebSocket closed with code: {}, reason: {}",
                    close.code, close.reason
                );
                break; // Exit the read loop on close
            }
            Ok(Message::Close(None)) => {
                info!("({ws_text}) WebSocket closed without close frame");
                break; // Exit the read loop on close
            }
            Ok(Message::Pong(_bytes)) => {}
            Ok(_) => {} // This message type is not handled
            Err(e) => {
                error!("({ws_text}) Error receiving message: {:?}", e);
                break; // Exit the read loop on error
            }
        }
    }
}
