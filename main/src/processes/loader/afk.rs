use std::sync::Arc;

use framework::{
    data,
    global::{GlobalMap, UserGlobalType},
    processes::ProcessLoop,
};
use futures_util::StreamExt;
use serenity::{async_trait, prelude::TypeMapKey};
use utils::{error, info};

use crate::global::{afk::AfkStatus, backend_http::BackendHttp};

#[derive(Debug, Clone)]
pub struct AfkLoader;

impl TypeMapKey for AfkLoader {
    type Value = Arc<AfkLoader>;
}

#[async_trait]
impl ProcessLoop for AfkLoader {
    async fn process(&self, _: utils::HttpType, data: utils::DataType) {
        let (backend_http, afk_statuses) = {
            let data = data.read().await;
            let Some(backend_http) = data.get::<BackendHttp>().cloned() else {
                error!("AFKLoader: BackendHttp not found in data");
                return;
            };

            let Some(afk_statuses) = data.get::<GlobalMap<AfkStatus>>().cloned() else {
                error!("AFKLoader: GlobalMap<AfkStatus> not found in data");
                return;
            };
            (backend_http, afk_statuses)
        };

        let mut response_stream = backend_http.stream::<AfkStatus>("api/afk");
        let mut count = 0;
        while let Some(afk_status) = response_stream.next().await {
            match afk_status {
                Ok(afk_status) => {
                    let key = afk_status
                        .guild_id
                        .map(|g_id| UserGlobalType::Guild(g_id, afk_status.user_id))
                        .unwrap_or(UserGlobalType::User(afk_status.user_id));
                    afk_statuses.insert(key, afk_status).await;
                    count += 1;
                }
                Err(e) => {
                    error!("AFKLoader: Error reading AFK status from stream: {e}");
                    continue;
                }
            }
        }

        info!("AfkLoader: Finished loading AFK statuses, total count: {count}");
    }
}
