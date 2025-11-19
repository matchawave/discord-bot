use serde::{Deserialize, Serialize};
use serenity::all::Colour;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    colour: Colour,
    language: Language,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Language {
    English,
    French,
    Spanish,
}
