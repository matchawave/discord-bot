use std::collections::HashMap;

use framework::{DataExtract, DataExtractable, Extractable, extractors::Extractor};
use serenity::{all::GuildId, prelude::TypeMapKey};
use utils::{DataType, Pointer};

#[derive(Clone, Default, DataExtractable, DataExtract)]
pub struct States(Pointer<HashMap<GuildId, DataType>>);

impl TypeMapKey for States {
    type Value = Pointer<HashMap<GuildId, DataType>>;
}

impl States {}
