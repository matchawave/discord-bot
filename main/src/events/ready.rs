use serenity::all::ShardId;
use utils::{error, info};

use crate::global::backend_http::BackendHttp;

pub async fn shard_started(shard_id: ShardId, backend_http: BackendHttp) {
    info!("Shard {} is ready", shard_id);
    let path = format!("api/shard/{}", shard_id);
    if let Err(e) = backend_http.post::<_, ()>(&path, &()).await {
        error!("Failed to post shard started event: {}", e);
    }
}
