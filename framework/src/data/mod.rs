mod cooldown;
mod ephemeral;
mod guild;
mod prefix;

pub use cooldown::*;
pub use ephemeral::*;
pub use guild::*;
pub use prefix::*;

use std::sync::Arc;

use serenity::prelude::{TypeMap, TypeMapKey};

use crate::{Extractable, sharded_data};

struct Datas;
impl TypeMapKey for Datas {
    type Value = Arc<Vec<Arc<TypeMap>>>;
}

sharded_data!(Data, Datas, { set_sharded_data });
pub fn set_sharded_data(data: &mut serenity::prelude::TypeMap) {
    Prefixes::init(data);
    Guilds::init(data);
    Cooldowns::init(data);
    Ephemerals::init(data);
}
