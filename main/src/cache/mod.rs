use framework::Extractable;
use serenity::prelude::TypeMap;

use crate::cache::snipe::{EditSnipes, ReactionSnipes, Snipes};
pub mod snipe;

pub fn set_sharded_cache(shards: usize, data: &mut TypeMap) {
    // Cache::initialize(shards, data, |map| {
    //     Snipes::init(map);
    //     EditSnipes::init(map);
    //     ReactionSnipes::init(map);
    // });
}
