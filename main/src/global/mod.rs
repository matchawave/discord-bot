pub mod afk;
pub mod backend_http;
use framework::global::{GlobalCache, GlobalMap};
use serenity::prelude::TypeMap;

use crate::configs::voice::VoiceConfig;
use afk::AfkStatus;
use backend_http::BackendHttp;
pub fn set_global(backend_http: BackendHttp, data: &mut TypeMap) {
    data.insert::<GlobalCache<VoiceConfig>>(GlobalCache::default());
    data.insert::<GlobalMap<afk::AfkStatus>>(GlobalMap::default());

    data.insert::<BackendHttp>(backend_http);
}
