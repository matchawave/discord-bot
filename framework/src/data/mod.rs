pub mod cooldown;
pub mod ephemeral;
pub mod guild;

use std::sync::Arc;

use serenity::prelude::{TypeMap, TypeMapKey};

use crate::{data::guild::Guilds, sharded_data};

struct Datas;
impl TypeMapKey for Datas {
    type Value = Arc<Vec<Arc<TypeMap>>>;
}

sharded_data!(Data, Datas, { set_sharded_data });
pub fn set_sharded_data(data: &mut serenity::prelude::TypeMap) {
    Guilds::init(data);
}

pub trait DataExt {
    fn init(map: &mut TypeMap);
    fn retrieve(map: &Arc<TypeMap>) -> Self
    where
        Self: Sized;
}
