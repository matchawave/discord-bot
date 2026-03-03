use core::fmt;
use std::{pin::Pin, sync::Arc};

use framework::extractors::{ContextExtractor, Extractor};
use futures_util::{Stream, StreamExt};
use reqwest::{Client, ClientBuilder, Error, StatusCode, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};

use serenity::{all::Context, async_trait};
use utils::{Pointer, error, info};

const PROTOCOL: &str = "http";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BackendHttpError {
    Request(String),
    Status(String),
    NotJson(String),
    Parse(String),
    Stream(String),
}

impl BackendHttpError {
    pub fn request(method: &str, endpoint: &str, error: Error) -> Self {
        BackendHttpError::Request(format!("{method} ({endpoint})\n{error:?}"))
    }

    pub fn status(method: &str, endpoint: &str, status: StatusCode, response: String) -> Self {
        BackendHttpError::Status(format!(
            "{method} ({endpoint}): Status {status}\nBody: {response}"
        ))
    }

    pub fn not_json(method: &str, endpoint: &str, error: Error) -> Self {
        BackendHttpError::NotJson(format!(
            "{method} ({endpoint}): Response was not valid JSON\n{error:?}"
        ))
    }

    pub fn parse(method: &str, endpoint: &str, error: serde_json::error::Error) -> Self {
        BackendHttpError::Parse(format!(
            "{method} ({endpoint}): Failed to parse JSON\n{error:?}"
        ))
    }

    pub fn stream(endpoint: &str, reason: &str, error: Error) -> Self {
        BackendHttpError::Stream(format!("({endpoint}): {reason} \n{error:?}"))
    }
}

impl fmt::Display for BackendHttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendHttpError::Request(e) => write!(f, "Request error:{}", e),
            BackendHttpError::Status(e) => write!(f, "Status error: {}", e),
            BackendHttpError::Parse(e) => write!(f, "Parse: {}", e),
            BackendHttpError::NotJson(e) => write!(f, "Not JSON: {}", e),
            BackendHttpError::Stream(e) => write!(f, "Stream {}", e),
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

    pub async fn get<U: DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<Option<U>, BackendHttpError> {
        let url = self.get_link(endpoint);
        let response = (self.client.get(&url).send().await)
            .map_err(|e| BackendHttpError::request("GET", endpoint, e))?;
        if response.status().is_server_error() || response.status().is_client_error() {
            let status = response.status();
            let response = (response.text().await)
                .unwrap_or_else(|_| "Unable to read response body".to_string());
            return Err(BackendHttpError::status("GET", endpoint, status, response));
        }
        let body: String =
            (response.text().await).map_err(|e| BackendHttpError::not_json("GET", endpoint, e))?;

        let trimmed_body = body.trim().to_lowercase();
        if trimmed_body.is_empty() || trimmed_body == "null" {
            return Ok(None);
        }

        let parsed = serde_json::from_str::<U>(&body)
            .map_err(|e| BackendHttpError::parse("GET", endpoint, e))?;

        Ok(Some(parsed))
    }

    pub async fn post<T: Serialize, U: DeserializeOwned>(
        &self,
        endpoint: &str,
        payload: &T,
    ) -> Result<Option<U>, BackendHttpError> {
        let url = self.get_link(endpoint);
        match self.client.post(&url).json(payload).send().await {
            Ok(resp) => {
                if resp.status().is_server_error() || resp.status().is_client_error() {
                    let status = resp.status();
                    let response = (resp.text().await)
                        .unwrap_or_else(|_| "Unable to read response body".to_string());
                    return Err(BackendHttpError::status("POST", endpoint, status, response));
                }

                let body = (resp.text().await)
                    .map_err(|e| BackendHttpError::not_json("POST", endpoint, e))?;

                let trimmed_body = body.trim().to_lowercase();
                if trimmed_body.is_empty() || trimmed_body == "null" {
                    return Ok(None);
                }

                let parsed = serde_json::from_str::<U>(&body)
                    .map_err(|e| BackendHttpError::parse("POST", endpoint, e))?;

                Ok(Some(parsed))
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
                    return Err(BackendHttpError::status(
                        "DELETE", endpoint, status, response,
                    ));
                }
                let body = (resp.text().await)
                    .map_err(|e| BackendHttpError::not_json("DELETE", endpoint, e))?;

                let trimmed_body = body.trim().to_lowercase();
                if trimmed_body.is_empty() || trimmed_body == "null" {
                    return Ok(None);
                }

                let parsed = serde_json::from_str::<U>(&body)
                    .map_err(|e| BackendHttpError::parse("DELETE", endpoint, e))?;

                Ok(Some(parsed))
            }
            Err(e) => Err(BackendHttpError::Request(format!(
                "Unable to DELETE {endpoint}: {e:?}",
            ))),
        }
    }

    pub async fn stream<U>(
        &self,
        endpoint: &str,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<U, BackendHttpError>> + Send + Sync + '_>>,
        BackendHttpError,
    >
    where
        U: DeserializeOwned + Send + Sync + 'static,
    {
        let link = self.get_link(endpoint);
        let response = (self.client.get(&link).send().await)
            .map_err(|e| BackendHttpError::request("GET", endpoint, e))?;

        if response.status().is_server_error() || response.status().is_client_error() {
            let status = response.status();
            let response = (response.text().await)
                .unwrap_or_else(|_| "Unable to read response body".to_string());
            return Err(BackendHttpError::status("GET", endpoint, status, response));
        }
        let endpoint = endpoint.to_string();
        let stream = async_stream::stream! {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        yield Err(BackendHttpError::stream(&endpoint, "Failed to read chunk", e));
                        continue;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(newline_pos) = buffer.find('\n') { // Data is delimited by newlines
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.is_empty() { // Skip empty lines
                        continue;
                    }

                    match serde_json::from_str::<U>(&line) {
                        Ok(item) => yield Ok(item),
                        Err(e) => {
                            error!("Failed to parse line from {}: {}\nLine content: {}", link, e, line);
                            yield Err(BackendHttpError::parse("Stream", &endpoint, e));
                        }
                    }
                }
            }
            if !buffer.trim().is_empty() { // Handle any remaining data in the buffer after the stream ends
                match serde_json::from_str::<U>(buffer.trim()) {
                    Ok(item) => yield Ok(item),
                    Err(e) => {
                        error!("Failed to parse final buffer from {}: {}", link, e);
                        yield Err(BackendHttpError::parse("Stream", &endpoint, e));
                    }
                }
            }
        };
        Ok(Box::pin(stream))
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

pub async fn set_shards(backend_http: &BackendHttp, count: u32) {
    let path = format!("api/shards/started/{}", count);
    match backend_http.post::<(), ()>(&path, &()).await {
        Ok(_) => info!("Set shard count to {}", count),
        Err(e) => error!("Failed to set shard count to {}: {}", count, e),
    }
}
