mod aliases;
mod cooldown;
mod ephemeral;
mod prefix;

pub use aliases::*;
pub use cooldown::*;
pub use ephemeral::*;
pub use prefix::*;

use std::sync::Arc;

use serenity::prelude::{TypeMap, TypeMapKey};

use crate::{Extractable, guilds::Guilds, sharded_data};

struct Datas;
impl TypeMapKey for Datas {
    type Value = Arc<Vec<Arc<TypeMap>>>;
}

sharded_data!(Data, Datas, { set_sharded_data });
pub fn set_sharded_data(data: &mut serenity::prelude::TypeMap) {
    Guilds::init(data);
}
