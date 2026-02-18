use std::sync::Arc;

use framework::extractors::{ContextExtractor, Extractor};
use reqwest::{Client, ClientBuilder, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use serenity::{
    all::{Context, GuildId, ShardId, UserId},
    async_trait,
};
use utils::{Pointer, error, info};

use crate::global::afk::AfkStatus;

const PROTOCOL: &str = "http";

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

    pub async fn get<U: DeserializeOwned>(&self, endpoint: &str) -> Option<U> {
        let url = self.get_link(endpoint);
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<U>().await {
                Ok(data) => Some(data),
                Err(e) => {
                    println!("Error parsing GET response from {}: {:?}", url, e);
                    None
                }
            },
            Err(e) => {
                println!("Error making GET request to {}: {:?}", url, e);
                None
            }
        }
    }

    pub async fn post<T: Serialize, U: DeserializeOwned>(
        &self,
        endpoint: &str,
        payload: &T,
    ) -> Option<U> {
        let url = self.get_link(endpoint);
        println!("POST to URL: {}", url);
        match self.client.post(&url).json(payload).send().await {
            Ok(resp) => match resp.json::<U>().await {
                Ok(data) => Some(data),
                Err(e) => {
                    println!("Error parsing POST response from {}: {:?}", url, e);
                    None
                }
            },
            Err(e) => {
                println!("Error making POST request to {}: {:?}", url, e);
                None
            }
        }
    }

    pub async fn delete<U: DeserializeOwned>(&self, endpoint: &str) -> Option<U> {
        let url = self.get_link(endpoint);
        match self.client.delete(&url).send().await {
            Ok(resp) => match resp.json::<U>().await {
                Ok(data) => Some(data),
                Err(e) => {
                    println!("Error parsing DELETE response from {}: {:?}", url, e);
                    None
                }
            },
            Err(e) => {
                println!("Error making DELETE request to {}: {:?}", url, e);
                None
            }
        }
    }

    pub async fn put<T: Serialize, U: DeserializeOwned>(
        &self,
        endpoint: &str,
        payload: &T,
    ) -> Option<U> {
        let url = self.get_link(endpoint);
        match self.client.put(&url).json(payload).send().await {
            Ok(resp) => match resp.json::<U>().await {
                Ok(data) => Some(data),
                Err(e) => {
                    println!("Error parsing PUT response from {}: {:?}", url, e);
                    None
                }
            },
            Err(e) => {
                println!("Error making PUT request to {}: {:?}", url, e);
                None
            }
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

    pub async fn set_user_afk(
        &self,
        user_id: UserId,
        guild_id: Option<GuildId>,
        reason: String,
    ) -> Result<AfkStatus, String> {
        let path = format!("api/afk/user/{}", user_id);
        let link = self.get_link(&path);
        let body = json!({
            "guild_id": guild_id.map(|g| g.to_string()),
            "reason": reason,
        });

        match self.client.post(&link).json(&body).send().await {
            Ok(r) => {
                if r.status().is_server_error() || r.status().is_client_error() {
                    return Err(format!("Failed to set AFK status: {}", r.status()));
                }

                match r.json::<AfkStatus>().await {
                    Ok(afk_status) => Ok(afk_status),
                    Err(e) => Err(format!(
                        "Error parsing AFK status response from {}: {:?}",
                        link, e
                    )),
                }
            }
            Err(e) => Err(format!("Error making request to {}: {:?}", link, e)),
        }
    }

    pub async fn remove_user_afk(
        &self,
        user_id: UserId,
        guild_id: Option<GuildId>,
    ) -> Result<(), String> {
        let mut path = format!("api/afk/user/{}", user_id);
        if let Some(guild_id) = guild_id {
            path.push_str(&format!("?guild_id={}", guild_id));
        }

        let link = self.get_link(&path);
        match self.client.delete(&link).send().await {
            Ok(r) => {
                if r.status().is_server_error() || r.status().is_client_error() {
                    return Err(format!("Failed to remove AFK status: {}", r.status()));
                }
            }
            Err(e) => return Err(format!("Error making request to {}: {:?}", link, e)),
        }
        Ok(())
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
