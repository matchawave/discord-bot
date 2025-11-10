use std::collections::HashMap;

use serenity::{
    all::{CommandDataOption, CommandDataOptionValue, Context},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::{command::CommandAction, extractors::Extractor};

pub struct InteractionOptions(HashMap<String, CommandDataOptionValue>);

impl InteractionOptions {
    pub fn get(&self, key: &str) -> Option<&CommandDataOptionValue> {
        self.0.get(key)
    }
}

#[async_trait]
impl Extractor<CommandAction> for Vec<String> {
    async fn extract(_ctx: &Context, ev: &CommandAction, _p: &Pointer<Parser>) -> Option<Self> {
        if let CommandAction::Message(msg) = ev {
            return Some(
                msg.content
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect(),
            );
        };
        None
    }
}

#[async_trait]
impl Extractor<CommandAction> for InteractionOptions {
    async fn extract(_ctx: &Context, ev: &CommandAction, _p: &Pointer<Parser>) -> Option<Self> {
        if let CommandAction::Interaction(interaction) = ev {
            let mut map = HashMap::new();

            for option in &interaction.data.options {
                map.insert(option.name.clone(), option.value.clone());
            }

            return Some(InteractionOptions(map));
        };
        None
    }
}

impl From<Vec<CommandDataOption>> for InteractionOptions {
    fn from(options: Vec<CommandDataOption>) -> Self {
        let mut map = HashMap::new();

        for option in options {
            map.insert(option.name, option.value);
        }

        InteractionOptions(map)
    }
}
