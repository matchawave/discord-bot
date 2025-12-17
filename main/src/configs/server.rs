use serde::{Deserialize, Serialize};
use serenity::{all::Colour, prelude::TypeMapKey};
use utils::Pointer;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    colour: Colour,
    language: Language,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            colour: Colour::BLITZ_BLUE,
            language: Language::default(),
        }
    }
}

impl TypeMapKey for ServerConfig {
    type Value = Pointer<ServerConfig>;
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum Language {
    #[default]
    English,
    French,
    Spanish,
}
