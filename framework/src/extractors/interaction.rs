use serenity::{
    all::{CommandInteraction, Context},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::{
    command::{CommandAction, CommandEvent},
    extractors::Extractor,
};

#[async_trait]
impl Extractor<CommandEvent> for CommandInteraction {
    async fn extract(_ctx: &Context, action: &CommandEvent, _p: &Pointer<Parser>) -> Option<Self> {
        match &action.action {
            CommandAction::Interaction(interaction) => Some(*interaction.clone()),
            _ => None,
        }
    }
}
