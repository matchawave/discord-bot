use framework::{
    build_process, extractors::ContextExtractor, global::GlobalMap, processes::ProcessLoop,
};
use futures_util::StreamExt;
use utils::{error, info};

use crate::global::backend_http::BackendHttp;

use chrono::DateTime;
use serde::Deserialize;
use serenity::all::{GuildId, UserId};
use utils::{deserialize_date, deserialize_id, deserialize_optional_id};

#[derive(Deserialize, Clone, Debug)]
pub struct AfkStatus {
    #[serde(deserialize_with = "deserialize_id")]
    pub user_id: UserId,
    #[serde(deserialize_with = "deserialize_optional_id")]
    pub guild_id: Option<GuildId>,
    #[serde(deserialize_with = "deserialize_date")]
    pub created_at: DateTime<chrono::Utc>,
    pub reason: String,
}

build_process!(AfkInstance, GlobalMap<AfkStatus>);

#[serenity::async_trait]
impl ProcessLoop for AfkInstance {
    async fn process(&self, _: utils::HttpType, data: utils::DataType) {
        let backend_http = {
            let data = data.read().await;
            let Some(backend_http) = data.get::<BackendHttp>().cloned() else {
                error!("AFKInstance: BackendHttp not found in data");
                return;
            };
            backend_http
        };

        match backend_http.api().stream::<AfkStatus>("afk").await {
            Ok(mut stream) => {
                let mut count = 0;
                while let Some(afk_status) = stream.next().await {
                    match afk_status {
                        Ok(status) => {
                            let mut afk_statuses = self.0.write().await;
                            (afk_statuses.insert(status.guild_id, status.user_id, status));
                            count += 1;
                        }
                        Err(e) => {
                            error!("AFKInstance: Error reading AFK status from stream\n{e}");
                            continue;
                        }
                    }
                }
                info!("AfkInstance: Finished loading AFK statuses, total count: {count}");
            }
            Err(e) => {
                error!("AFKInstance: Failed to start AFK status stream\n{e}");
            }
        }
    }
}
