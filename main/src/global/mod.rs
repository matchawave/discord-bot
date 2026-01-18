pub mod backend_http;

use framework::global::GlobalMap;
use serenity::prelude::TypeMap;

use crate::configs::voice::VoiceConfig;
use backend_http::BackendHttp;
pub fn set_global(backend_http: BackendHttp, data: &mut TypeMap) {
    data.insert::<GlobalMap<VoiceConfig>>(GlobalMap::default());
    data.insert::<BackendHttp>(backend_http);
}
