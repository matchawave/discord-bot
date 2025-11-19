use framework::cache::Cache;
use serenity::prelude::TypeMap;

pub fn set_sharded_cache(shards: usize, data: &mut TypeMap) {
    Cache::initialize(shards, data, |map| {});
}
