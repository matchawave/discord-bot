use core::fmt;
use std::{pin::Pin, sync::Arc};

use framework::extractors::{ContextExtractor, Extractor};
use futures_util::{Stream, StreamExt};
use reqwest::{Client, ClientBuilder, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};

use serenity::{
    all::{Context, GuildId, ShardId},
    async_trait,
};
use utils::{Pointer, error, info};

const PROTOCOL: &str = "http";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BackendHttpError {
    Request(String),
    Response(String),
    Parse(String),
}

impl fmt::Display for BackendHttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendHttpError::Request(e) => write!(f, "Request error:\n{}", e),
            BackendHttpError::Response(e) => write!(f, "Response error:\n{}", e),
            BackendHttpError::Parse(e) => write!(f, "Parse error:\n{}", e),
        }
    }
}

#[derive(Clone)]
pub struct BackendHttp {
    pub client: Arc<Client>,
    pub api_url: String,
}

impl BackendHttp {
    pub fn new(token: &str, api_url: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("client", format!("DiscordBot {}", token).parse().unwrap());

        let client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();
        BackendHttp {
            client: Arc::new(client),
            api_url: api_url.to_string(),
        }
    }

    fn get_link<T: Into<String>>(&self, endpoint: T) -> String {
        format!("{}://{}/{}", PROTOCOL, self.api_url, endpoint.into())
    }

    pub async fn get<U: DeserializeOwned>(&self, endpoint: &str) -> Result<U, BackendHttpError> {
        let url = self.get_link(endpoint);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_server_error() || resp.status().is_client_error() {
                    let status = resp.status();
                    let response = (resp.text().await)
                        .unwrap_or_else(|_| "Unable to read response body".to_string());
                    return Err(BackendHttpError::Response(format!(
                        "Bad status from GET {endpoint}: {status}\nResponse body: {response}",
                    )));
                }
                match resp.json::<U>().await {
                    Ok(data) => Ok(data),
                    Err(e) => Err(BackendHttpError::Parse(format!(
                        "Incorrectly parsing GET {endpoint}: {e:?}",
                    ))),
                }
            }
            Err(e) => {
                println!("Error making GET request to {}: {:?}", url, e);
                Err(BackendHttpError::Request(format!(
                    "Unable to GET {}: {:?}",
                    endpoint, e
                )))
            }
        }
    }

    pub async fn post<T: Serialize, U: DeserializeOwned>(
        &self,
        endpoint: &str,
        payload: &T,
    ) -> Result<U, BackendHttpError> {
        let url = self.get_link(endpoint);
        match self.client.post(&url).json(payload).send().await {
            Ok(resp) => {
                if resp.status().is_server_error() || resp.status().is_client_error() {
                    let status = resp.status();
                    let response = (resp.text().await)
                        .unwrap_or_else(|_| "Unable to read response body".to_string());
                    return Err(BackendHttpError::Response(format!(
                        "Bad status from POST {endpoint}: {status}\nResponse body: {response}"
                    )));
                }
                match resp.json::<U>().await {
                    Ok(data) => Ok(data),
                    Err(e) => Err(BackendHttpError::Parse(format!(
                        "Incorrectly parsing POST {}: {:?}",
                        endpoint, e
                    ))),
                }
            }
            Err(e) => Err(BackendHttpError::Request(format!(
                "Unable to POST {}: {:?}",
                endpoint, e
            ))),
        }
    }

    pub async fn delete<U: DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<Option<U>, BackendHttpError> {
        let url = self.get_link(endpoint);
        match self.client.delete(&url).send().await {
            Ok(resp) => {
                if resp.status().is_server_error() || resp.status().is_client_error() {
                    let status = resp.status();
                    let response = (resp.text().await)
                        .unwrap_or_else(|_| "Unable to read response body".to_string());
                    return Err(BackendHttpError::Response(format!(
                        "Bad status from DELETE {endpoint}: {status}\nResponse body: {response}"
                    )));
                }
                let body = resp.text().await.map_err(|e| {
                    BackendHttpError::Parse(format!(
                        "Error reading DELETE response from {url}: {e:?}",
                    ))
                })?;

                if body.is_empty() || body.contains('[') || body.contains('{') {
                    // crude check for empty response vs JSON response
                    Ok(None)
                } else {
                    match serde_json::from_str::<U>(&body) {
                        Ok(data) => Ok(Some(data)),
                        Err(e) => Err(BackendHttpError::Parse(format!(
                            "Error parsing DELETE response from {url}: {e:?}",
                        ))),
                    }
                }
            }
            Err(e) => Err(BackendHttpError::Request(format!(
                "Unable to DELETE {endpoint}: {e:?}",
            ))),
        }
    }

    pub async fn set_shards(&self, count: u32) {
        let path = format!("api/shards/started/{}", count);
        let link = self.get_link(&path);

        match self.client.post(&link).send().await {
            Ok(r) => {
                if r.status().is_server_error() || r.status().is_client_error() {
                    error!("Setting shards at {}: {:?}", link, r.status());
                } else {
                    info!("Successfully set shards at {}: {:?}", link, r.status());
                }
            }
            Err(e) => {
                error!("Error setting shards at {}: {:?}", link, e);
            }
        }
    }

    pub async fn register_guild(&self, guild_id: GuildId, shard_id: ShardId) {
        let path = format!("api/guild/{}?shard_id={}", guild_id, shard_id);
        let link = self.get_link(&path);

        match self.client.post(&link).send().await {
            Ok(r) => {
                if r.status().is_server_error() || r.status().is_client_error() {
                    error!("Registering guild at {}: {:?}", link, r.status());
                } else {
                    info!(
                        "Successfully registered guild at {}: {:?}",
                        link,
                        r.status()
                    );
                }
            }
            Err(e) => {
                error!("Error registering guild at {}: {:?}", link, e);
            }
        }
    }

    pub async fn disable_guild(&self, guild_id: GuildId) {
        let path = format!("api/guild/{}", guild_id);
        let link = self.get_link(&path);

        match self.client.delete(&link).send().await {
            Ok(r) => {
                if r.status().is_server_error() || r.status().is_client_error() {
                    error!("Disabling guild at {}: {:?}", link, r.status());
                } else {
                    info!("Successfully disabled guild at {}: {:?}", link, r.status());
                }
            }
            Err(e) => {
                error!("Error disabling guild at {}: {:?}", link, e);
            }
        }
    }

    pub async fn delete_guild(&self, guild_id: GuildId) {
        let path = format!("api/guilds/{}", guild_id);
        let link = self.get_link(&path);

        match self.client.delete(&link).send().await {
            Ok(r) => {
                if r.status().is_server_error() || r.status().is_client_error() {
                    error!("Deleting guild at {}: {:?}", link, r.status());
                } else {
                    info!("Successfully deleted guild at {}: {:?}", link, r.status());
                }
            }
            Err(e) => {
                error!("Error deleting guild at {}: {:?}", link, e);
            }
        }
    }

    pub fn stream<T>(
        &self,
        endpoint: &str,
    ) -> Pin<Box<dyn Stream<Item = Result<T, String>> + Send + Sync + '_>>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        let link = self.get_link(endpoint);
        Box::pin(async_stream::stream! {
            let response = match self.client.get(&link).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Error making request to {}: {:?}", link, e);
                        yield Err(format!("Error making request to {}: {:?}", link, e));
                        return;
                }
            };

            if response.status().is_server_error() || response.status().is_client_error() {
                let status = response.status();
                yield Err(format!("Error making request to {}: {:?}", link, status));
                return;
            }
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        yield Err(format!("Error reading chunk: {}", e));
                        continue;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    match serde_json::from_str::<T>(&line) {
                        Ok(item) => yield Ok(item),
                        Err(e) => {
                            error!("Failed to parse NDJSON line from {}: {} | Line: {}", link, e, line);
                        }
                    }
                }
            }
            if !buffer.trim().is_empty() {
                match serde_json::from_str::<T>(buffer.trim()) {
                    Ok(item) => yield Ok(item),
                    Err(e) => {
                        error!("Failed to parse final buffer from {}: {}", link, e);
                    }
                }
            }
        })
    }
}

impl serenity::prelude::TypeMapKey for BackendHttp {
    type Value = BackendHttp;
}

#[async_trait]
impl ContextExtractor for BackendHttp {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        let data = ctx.data.read().await;
        data.get::<BackendHttp>().cloned()
    }
}

#[async_trait]
impl<T> Extractor<T> for BackendHttp
where
    T: Send + Sync,
{
    async fn extract(
        ctx: &serenity::all::Context,
        _: &T,
        _: &Pointer<utils::Parser>,
    ) -> Option<Self> {
        BackendHttp::extract_context(ctx).await
    }
}
