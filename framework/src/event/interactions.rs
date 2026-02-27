use serenity::all::{Context, Interaction};
use utils::{Parser, Pointer, ResponseError};

use crate::{
    command::{CommandEvent, CommandExecution, CommandManager},
    extractors::ContextExtractor,
};

pub async fn handle(
    ctx: &Context,
    interaction: Interaction,
) -> Result<Option<(CommandEvent, Pointer<Parser>)>, ResponseError> {
    let Interaction::Command(command) = interaction else {
        return Ok(None);
    };

    let Some(command_manager) = CommandManager::extract_context(ctx).await else {
        return Err(ResponseError::Err(
            "CommandManager not found in TypeMap".into(),
        ));
    };

    command_manager.execute(ctx, command.clone()).await
}
