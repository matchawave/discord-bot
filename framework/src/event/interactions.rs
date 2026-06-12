use serenity::all::{Context, Interaction};
use utils::{Parser, Pointer, ResponseError};

use crate::{
    command::{CommandEvent, CommandExecution},
    global::Commands,
};

pub async fn handle(
    ctx: &Context,
    interaction: Interaction,
) -> Result<Option<(CommandEvent, Pointer<Parser>)>, ResponseError> {
    let Interaction::Command(command) = interaction else {
        return Ok(None);
    };

    let command_manager = {
        let data = ctx.data.read().await;
        let Some(command_manager) = data.get::<Commands>() else {
            return Err(ResponseError::new("CommandManager not found in TypeMap"));
        };
        command_manager.clone()
    };

    command_manager.execute(ctx, command.clone()).await
}
