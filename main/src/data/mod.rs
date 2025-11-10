use framework::{
    cache::Cache,
    data::{Data, DataExt, cooldown::Cooldowns, ephemeral::Ephemerals},
    extractors::Prefix,
};

use crate::data::{
    member::MembersInfo,
    snipe::{EditSnipes, Snipes},
};

pub mod member;
pub mod snipe;
pub mod voice_master;

pub fn set_sharded_data(shards: usize, data: &mut serenity::prelude::TypeMap) {
    Data::initialize(shards, data, |map| {
        Cooldowns::init(map);
        Ephemerals::init(map);
        Snipes::init(map);
        EditSnipes::init(map);
        MembersInfo::init(map);
    });
    Cache::set(shards, data);
    Prefix::set(data);
}
