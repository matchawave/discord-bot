use framework::{DataExtractable, data::Data};
use serenity::prelude::TypeMap;

use crate::data::{
    member::MembersInfo,
    snipe::{EditSnipes, Snipes},
    voice_channels::ChannelMembers,
    voice_master::VoiceMasters,
};

pub mod member;
pub mod snipe;
pub mod voice_channels;
pub mod voice_master;

pub fn set_sharded_data(shards: usize, data: &mut TypeMap) {
    Data::initialize(shards, data, |map| {
        Snipes::init(map);
        EditSnipes::init(map);
        MembersInfo::init(map);
        VoiceMasters::init(map);
        ChannelMembers::init(map);
    });
}
