mod builder;
mod configs;
mod misc;
mod reader;
mod writer;

use serde::de::DeserializeOwned;
pub use writer::WebSocketWriter;

use std::{collections::HashMap, sync::Arc};

use colored::Colorize;

use serenity::{all::ApplicationId, async_trait, futures::StreamExt, prelude::TypeMapKey};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::ClientRequestBuilder};
use utils::{DataType, HttpType, debug, error, info};

use crate::{handler::HandlerFn, processes::ProcessLoop};

const PROTOCOL: &str = "ws";

type WebSocketStreamType = WebSocketStream<MaybeTlsStream<TcpStream>>;

type EventCallback = Box<dyn builder::WSDynHandler>;

#[derive(Default)]
pub struct WebSocketProcessorBuilder<T>(HashMap<T, Vec<EventCallback>>)
where
    T: std::cmp::Eq + std::hash::Hash + DeserializeOwned;

impl<T> WebSocketProcessorBuilder<T>
where
    T: std::cmp::Eq + std::hash::Hash + DeserializeOwned,
{
    pub fn add_callback<F, U>(mut self, event: T, callback: F) -> Self
    where
        F: HandlerFn<(U, DataType, HttpType), ()> + Send + Copy + Sync + 'static,
        U: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let handler = builder::WSHandlerBuilder::<U>::build(callback);
        self.0.entry(event).or_default().push(Box::new(handler));
        self
    }

    pub fn build(
        self,
        api_url: &str,
        user_id: ApplicationId,
        token: &str,
    ) -> WebSocketProcessor<T> {
        let ws_url = format!("{}://{}/api/gateway/{}", PROTOCOL, api_url, user_id);
        let req = configs::create_request(ws_url, token);
        WebSocketProcessor {
            req,
            callbacks: self.0,
        }
    }
}

pub struct WebSocketProcessor<T>
where
    T: DeserializeOwned + Eq + std::hash::Hash,
{
    req: ClientRequestBuilder,
    callbacks: HashMap<T, Vec<EventCallback>>,
}

impl<T> WebSocketProcessor<T>
where
    T: DeserializeOwned + Eq + std::hash::Hash,
{
    async fn connect(&self) -> Result<(writer::SocketWriter, reader::SocketReader), String> {
        let config = configs::create_config();

        let connector = configs::create_connector();

        let connection = tokio_tungstenite::connect_async_tls_with_config(
            self.req.clone(),
            Some(config),
            false,
            Some(connector),
        );
        match connection.await {
            Ok((ws_stream, _response)) => Ok(ws_stream.split()),
            Err(e) => Err(format!("Failed to establish WebSocket connection: {:?}", e)),
        }
    }
}

impl<T> TypeMapKey for WebSocketProcessor<T>
where
    T: DeserializeOwned + Eq + std::hash::Hash + Send + Sync + 'static,
{
    type Value = Arc<WebSocketProcessor<T>>;
}

#[async_trait]
impl<T> ProcessLoop for WebSocketProcessor<T>
where
    T: DeserializeOwned + Eq + std::hash::Hash + Default + Send + Sync + 'static,
{
    async fn process(&self, http: HttpType, data: DataType) {
        // Here you would implement the WebSocket connection logic
        let ws_text = "websocket".yellow();
        info!("({ws_text}) Starting WebSocket connection");
        let mut printed_reconnection = false;
        loop {
            match self.connect().await {
                Ok((writer, reader)) => {
                    let data = data.clone();
                    printed_reconnection = false;
                    {
                        let mut data_write = data.write().await;
                        data_write.insert::<writer::WebSocketWriter>(Arc::new(Mutex::new(writer)));
                        debug!("({ws_text}) WebSocket writer stored in data");
                    }
                    reader::read_socket(reader, &self.callbacks, http.clone(), data.clone()).await;
                    {
                        let mut data_write = data.write().await;
                        data_write.remove::<writer::WebSocketWriter>();
                        debug!("({ws_text}) WebSocket writer removed from data");
                    }
                    error!("({ws_text}) WebSocket connection closed, reconnecting...");
                }
                Err(e) => {
                    if !printed_reconnection {
                        printed_reconnection = true;
                        error!(
                            "({ws_text}) Failed to connect, retrying in 5 seconds...\n{}",
                            e
                        );
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}
