use serenity::all::{Context, Message, MessageUpdateEvent};
use utils::{Parser, Pointer, ResponseError};

use crate::{
    command::{CommandEvent, CommandExecution, CommandManager},
    extractors::ContextExtractor,
};

pub async fn handle_edited_command(
    ctx: &Context,
    updated_msg: &MessageUpdateEvent,
) -> Result<Option<(CommandEvent, Pointer<Parser>)>, ResponseError> {
    let mut msg = Message::default();
    updated_msg.apply_to_message(&mut msg);
    handle_command(ctx, msg).await
}

pub async fn handle_command(
    ctx: &Context,
    msg: Message,
) -> Result<Option<(CommandEvent, Pointer<Parser>)>, ResponseError> {
    if msg.author.bot {
        return Ok(None);
    }

    let Some(command_manager) = CommandManager::extract_context(ctx).await else {
        return Err(ResponseError::new("CommandManager not found in TypeMap"));
    };

    command_manager.execute(ctx, msg).await
}
