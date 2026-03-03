pub mod afk;
pub mod backend_http;
pub mod birthday;

use framework::global::{GlobalCache, GlobalMap};
use serenity::prelude::TypeMap;

use crate::configs::{AfkConfig, voice::VoiceConfig};
use afk::AfkStatus;
use backend_http::BackendHttp;
use birthday::Birthday;

pub fn set_global(backend_http: BackendHttp, data: &mut TypeMap) {
    data.insert::<GlobalCache<VoiceConfig>>(GlobalCache::default());
    data.insert::<GlobalCache<AfkConfig>>(GlobalCache::default());
    data.insert::<GlobalCache<Birthday>>(GlobalCache::default());
    data.insert::<GlobalMap<AfkStatus>>(GlobalMap::default());

    data.insert::<BackendHttp>(backend_http);
}
