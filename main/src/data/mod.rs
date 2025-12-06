use framework::{Extractable, ShardData, data::Data};
use serenity::prelude::TypeMap;

pub mod member_list;
pub mod state;

pub mod voice_master;

pub fn set_sharded_data(shards: usize, data: &mut TypeMap) {
    ShardData::init(shards, data);
    // Data::initialize(shards, data, |map| {
    //     ChannelMembers::init(map);
    //     States::init(map);
    // });
}
