use framework::{Extractable, data::Data};
use serenity::prelude::TypeMap;

use crate::data::{
    logs::LogConfigs, member::MembersInfo, voice_channels::ChannelMembers,
    voice_master::VoiceMasters,
};

pub mod logs;
pub mod member;

pub mod voice_channels;
pub mod voice_master;

pub fn set_sharded_data(shards: usize, data: &mut TypeMap) {
    Data::initialize(shards, data, |map| {
        MembersInfo::init(map);
        VoiceMasters::init(map);
        ChannelMembers::init(map);
        LogConfigs::init(map);
    });
}
