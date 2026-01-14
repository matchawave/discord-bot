pub mod backend_http;
pub mod shard_list;

use framework::global::GlobalMap;
use reqwest::Client;
use serenity::prelude::TypeMap;

use crate::configs::voice::VoiceConfig;
use backend_http::BackendHttp;
use shard_list::ShardList;
pub fn set_global(shards: usize, backend_http: BackendHttp, data: &mut TypeMap) {
    data.insert::<GlobalMap<VoiceConfig>>(GlobalMap::default());
    data.insert::<ShardList>(ShardList::build(shards).into());
    data.insert::<BackendHttp>(backend_http);
}
