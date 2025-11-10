use serenity::{
    all::{CommandInteraction, Context},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::{command::CommandAction, extractors::Extractor};

#[async_trait]
impl Extractor<CommandAction> for CommandInteraction {
    async fn extract(_ctx: &Context, action: &CommandAction, _p: &Pointer<Parser>) -> Option<Self> {
        match action {
            CommandAction::Interaction(interaction) => Some(*interaction.clone()),
            _ => None,
        }
    }
}
