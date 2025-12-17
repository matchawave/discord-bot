use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serenity::{all::UserId, prelude::TypeMapKey};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiNuke {
    pub whitelist: Vec<UserId>,
    #[serde_as(as = "Vec<(_, _)>")]
    pub modules: HashMap<AntiNukeProtection, AntiNukeConfig>,
}

impl TypeMapKey for AntiNuke {
    type Value = utils::Pointer<AntiNuke>;
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum AntiNukeProtection {
    MemberKick,
    MemberBan,
    Role,
    Webhook,
    Channel,
    Emoji,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiNukeConfig {
    punishment: String,
    threshold: u64,
    command: bool,
}
