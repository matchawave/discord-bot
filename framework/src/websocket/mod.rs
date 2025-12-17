use std::{os::windows::process, sync::Arc};

use colored::Colorize;
use serde::Serialize;
use serenity::{
    all::{Context, UserId},
    async_trait,
    futures::{
        SinkExt, StreamExt,
        stream::{SplitSink, SplitStream},
    },
    prelude::TypeMapKey,
};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{
    Connector, MaybeTlsStream, WebSocketStream,
    tungstenite::{ClientRequestBuilder, Message, Utf8Bytes, http::Uri, protocol::WebSocketConfig},
};
use utils::{DataType, HttpType, error, info};

use crate::{extractors::ContextExtractor, processes::ProcessLoop};

const PROTOCOL: &str = "ws";

/// Read buffer capacity. This buffer is eagerly allocated and used for receiving messages.
///
/// For high read load scenarios a larger buffer, e.g. 128 KiB, improves performance.
///
/// For scenarios where you expect a lot of connections and don't need high read load
/// performance a smaller buffer, e.g. 4 KiB, would be appropriate to lower total
/// memory usage.
///
/// The default value is 128 KiB.
const READ_BUFFER_SIZE: usize = 128 * 1024;

/// The target minimum size of the write buffer to reach before writing the data to the underlying stream.
/// The default value is 128 KiB.
///
/// If set to `0` each message will be eagerly written to the underlying stream.
/// It is often more optimal to allow them to buffer a little, hence the default value.
///
/// Note: [`flush`](WebSocket::flush) will always fully write the buffer regardless.
const WRITE_BUFFER_SIZE: usize = 128 * 1024;

/// The max size of the write buffer in bytes. Setting this can provide backpressure
/// in the case the write buffer is filling up due to write errors.
/// The default value is unlimited.
///
/// Note: The write buffer only builds up past [`write_buffer_size`](Self::write_buffer_size)
/// when writes to the underlying stream are failing. So the **write buffer can not
/// fill up if you are not observing write errors even if not flushing**.
///
/// Note: Should always be at least [`write_buffer_size + 1 message`](Self::write_buffer_size)
/// and probably a little more depending on error handling strategy.
const MAX_WRITE_BUFFER_SIZE: usize = usize::MAX;

/// The maximum size of an incoming message. `None` means no size limit. The default value is 64 MiB
/// which should be reasonably big for all normal use-cases but small enough to prevent
/// memory eating by a malicious user.
const MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024;

/// The maximum size of a single incoming message frame. `None` means no size limit. The limit is for
/// frame payload NOT including the frame header. The default value is 16 MiB which should
/// be reasonably big for all normal use-cases but small enough to prevent memory eating
/// by a malicious user.
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

/// When set to `true`, the server will accept and handle unmasked frames
/// from the client. According to the RFC 6455, the server must close the
/// connection to the client in such cases, however it seems like there are
/// some popular libraries that are sending unmasked frames, ignoring the RFC.
/// By default this option is set to `false`, i.e. according to RFC 6455.
const ACCEPT_UNMASKED_FRAMES: bool = false;

type WebSocketStreamType = WebSocketStream<MaybeTlsStream<TcpStream>>;
type SocketReader = SplitStream<WebSocketStreamType>;
type SocketWriter = SplitSink<WebSocketStreamType, Message>;

pub struct WebSocketProcessor {
    req: ClientRequestBuilder,
}

impl WebSocketProcessor {
    pub fn new(api_url: &str, user_id: UserId, token: &str) -> Self {
        let ws_url = format!("{}://{}/api/gateway/{}", PROTOCOL, api_url, user_id);
        let req = create_request(ws_url).with_header("client", format!("DiscordBot {}", token));
        Self { req }
    }

    async fn connect(&self) -> Option<(SocketWriter, SocketReader)> {
        let config = create_config();

        let connector = create_connector();

        let connection = tokio_tungstenite::connect_async_tls_with_config(
            self.req.clone(),
            Some(config),
            false,
            Some(connector),
        );
        let ws_text = "websocket".yellow();
        match connection.await {
            Ok((ws_stream, _response)) => {
                info!("({ws_text}) WebSocket connection established");
                Some(ws_stream.split())
            }
            Err(e) => {
                info!("({ws_text}) WebSocket connection error: {:?}", e);
                None
            }
        }
    }
}

#[async_trait]
impl ProcessLoop for WebSocketProcessor {
    async fn process(&self, http: HttpType, data: DataType) {
        // Here you would implement the WebSocket connection logic
        let ws_text = "websocket".yellow();
        info!("({ws_text}) Starting WebSocket connection");
        // Example: Establish connection using self.req

        loop {
            if let Some((writer, mut reader)) = self.connect().await {
                let data = data.clone();
                {
                    let mut data_write = data.write().await;
                    data_write.insert::<WebSocketWriter>(Arc::new(Mutex::new(writer)));
                }

                info!("({ws_text}) Handling WebSocket connection");

                // Example read loop
                while let Some(message) = reader.next().await {
                    match message {
                        Ok(msg) => {
                            info!("({ws_text}) Received message: {:?}", msg);
                            // Process the message here
                        }
                        Err(e) => {
                            error!("({ws_text}) Error receiving message: {:?}", e);
                            break; // Exit the read loop on error
                        }
                    }
                }

                info!("({ws_text}) WebSocket connection closed, reconnecting...");
            } else {
                error!("({ws_text}) Failed to connect, retrying in 5 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

fn create_request(ws_url: String) -> ClientRequestBuilder {
    let ws_text = "websocket".yellow();

    let uri = match Uri::try_from(&ws_url) {
        Ok(u) => u,
        Err(e) => {
            error!(
                "({ws_text}) Invalid WebSocket URI: {:?}, error: {:?}",
                ws_url, e
            );
            panic!("Invalid WebSocket URI");
        }
    };
    ClientRequestBuilder::new(uri)
}

fn create_config() -> WebSocketConfig {
    WebSocketConfig::default()
        .max_message_size(Some(MAX_MESSAGE_SIZE))
        .max_frame_size(Some(MAX_FRAME_SIZE))
        .accept_unmasked_frames(ACCEPT_UNMASKED_FRAMES)
        .read_buffer_size(READ_BUFFER_SIZE)
        .write_buffer_size(WRITE_BUFFER_SIZE)
        .max_write_buffer_size(MAX_WRITE_BUFFER_SIZE)
}

fn create_connector() -> Connector {
    Connector::Plain
}

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
