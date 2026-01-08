pub mod shard_list;

use framework::global::GlobalMap;
use serenity::prelude::TypeMap;

use crate::configs::voice::VoiceConfig;
use shard_list::ShardList;
pub fn set_global(shards: usize, data: &mut TypeMap) {
    data.insert::<GlobalMap<VoiceConfig>>(GlobalMap::default());
    data.insert::<ShardList>(ShardList::build(shards).into());
}
