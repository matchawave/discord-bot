use framework::{GlobalExtractable, global::UserConfigHash};
use serenity::prelude::TypeMap;

mod voice_configs;
pub use voice_configs::*;

pub fn set_global(data: &mut TypeMap) {
    UserConfigHash::<voice_configs::VoiceConfig>::init(data);
}
