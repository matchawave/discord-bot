use std::collections::HashMap;

use serenity::{all::GuildId, prelude::TypeMapKey};
use utils::{DataType, Pointer};

#[derive(Clone, Default)]
pub struct States(Pointer<HashMap<GuildId, DataType>>);

impl TypeMapKey for States {
    type Value = Pointer<HashMap<GuildId, DataType>>;
}

impl States {}
