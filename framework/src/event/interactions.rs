use serenity::all::{Context, Interaction};
use utils::error;

use crate::{
    command::{CommandExecution, CommandManager},
    extractors::ContextExtractor,
};

pub async fn handle(ctx: &Context, interaction: Interaction) -> Option<String> {
    let Interaction::Command(command) = interaction else {
        return None;
    };

    let Some(command_manager) = CommandManager::extract_context(ctx).await else {
        error!("CommandManager not found in TypeMap");
        return None;
    };

    command_manager.execute(ctx, command.clone()).await
}
