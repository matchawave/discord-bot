mod builder;
mod configs;
mod misc;
mod reader;
mod writer;

pub use misc::*;
pub use writer::WebSocketWriter;

use std::{collections::HashMap, sync::Arc};

use colored::Colorize;

use serenity::{all::ApplicationId, async_trait, futures::StreamExt, prelude::TypeMapKey};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::ClientRequestBuilder};
use utils::{DataType, HttpType, debug, error, info};

use crate::{HandlerFn, processes::ProcessLoop};

const PROTOCOL: &str = "ws";

type WebSocketStreamType = WebSocketStream<MaybeTlsStream<TcpStream>>;

type EventCallback = Box<dyn builder::WSDynHandler>;

#[derive(Default)]
pub struct WebSocketProcessorBuilder(HashMap<SocketReceiveEvent, Vec<EventCallback>>);

impl WebSocketProcessorBuilder {
    pub fn add_callback<F, T>(mut self, event: SocketReceiveEvent, callback: F) -> Self
    where
        F: HandlerFn<(T, DataType, HttpType), ()> + Send + Copy + Sync + 'static,
        T: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let handler = builder::WSHandlerBuilder::<T>::build(callback);
        self.0.entry(event).or_default().push(Box::new(handler));
        self
    }

    pub fn build(self, api_url: &str, user_id: ApplicationId, token: &str) -> WebSocketProcessor {
        let ws_url = format!("{}://{}/api/gateway/{}", PROTOCOL, api_url, user_id);
        let req = configs::create_request(ws_url, token);
        WebSocketProcessor {
            req,
            callbacks: self.0,
        }
    }
}

pub struct WebSocketProcessor {
    req: ClientRequestBuilder,
    callbacks: HashMap<SocketReceiveEvent, Vec<EventCallback>>,
}

impl WebSocketProcessor {
    async fn connect(&self) -> Option<(writer::SocketWriter, reader::SocketReader)> {
        let config = configs::create_config();

        let connector = configs::create_connector();

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

impl TypeMapKey for WebSocketProcessor {
    type Value = Arc<WebSocketProcessor>;
}

#[async_trait]
impl ProcessLoop for WebSocketProcessor {
    async fn process(&self, http: HttpType, data: DataType) {
        // Here you would implement the WebSocket connection logic
        let ws_text = "websocket".yellow();
        info!("({ws_text}) Starting WebSocket connection");
        // Example: Establish connection using self.req

        loop {
            if let Some((writer, reader)) = self.connect().await {
                let data = data.clone();
                {
                    let mut data_write = data.write().await;
                    data_write.insert::<writer::WebSocketWriter>(Arc::new(Mutex::new(writer)));
                    debug!("({ws_text}) WebSocket writer stored in data");
                }
                reader::read_socket(reader, &self.callbacks, http.clone(), data.clone()).await;

                error!("({ws_text}) WebSocket connection closed, reconnecting...");
            } else {
                error!("({ws_text}) Failed to connect, retrying in 5 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}
