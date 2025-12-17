use framework::global::GlobalMap;
use serenity::prelude::TypeMap;

use crate::configs::voice::VoiceConfig;

pub fn set_global(data: &mut TypeMap) {
    data.insert::<GlobalMap<VoiceConfig>>(GlobalMap::default());
}
