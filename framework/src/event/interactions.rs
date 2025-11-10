use serenity::all::{Context, Interaction};
use utils::{Parser, Pointer, error};

use crate::command::{CommandExecution, CommandManager};

pub async fn handle(
    ctx: &Context,
    interaction: &Interaction,
    parser: &Pointer<Parser>,
) -> Option<String> {
    let Interaction::Command(c) = interaction else {
        return None;
    };

    let command_manager = match CommandManager::get(&ctx.data).await {
        Some(manager) => manager,
        None => {
            error!("CommandManager not found in TypeMap");
            return None;
        }
    };

    command_manager.execute(ctx, c, parser).await
}
