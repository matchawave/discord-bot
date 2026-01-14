use std::sync::Arc;

use framework::extractors::{ContextExtractor, Extractor};
use reqwest::{Client, ClientBuilder, header::HeaderMap};
use serenity::{all::Context, async_trait};
use utils::{Parser, Pointer};

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

    // pub async fn
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
