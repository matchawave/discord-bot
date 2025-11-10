use serenity::all::{Context, Message, MessageUpdateEvent, PartialGuild};
use utils::{Parser, Pointer, error};

use crate::command::{CommandExecution, CommandManager};

pub async fn handle_edited_command(
    ctx: &Context,
    updated_msg: &MessageUpdateEvent,
    parser: &Pointer<Parser>,
) -> Option<String> {
    let mut msg = Message::default();
    updated_msg.apply_to_message(&mut msg);
    handle_command(ctx, &msg, parser).await
}

pub async fn handle_command(
    ctx: &Context,
    msg: &Message,
    parser: &Pointer<Parser>,
) -> Option<String> {
    if msg.author.bot {
        return None;
    }

    let command_manager = match CommandManager::get(&ctx.data).await {
        Some(manager) => manager,
        None => {
            error!("CommandManager not found in TypeMap");
            return None;
        }
    };

    command_manager.execute(ctx, msg, parser).await
}
