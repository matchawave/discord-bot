use serenity::all::{Context, Message, MessageUpdateEvent};
use utils::{Parser, Pointer, ResponseError, error};

use crate::{
    command::{CommandAction, CommandExecution, CommandManager},
    extractors::ContextExtractor,
};

pub async fn handle_edited_command(
    ctx: &Context,
    updated_msg: &MessageUpdateEvent,
) -> Result<Option<(String, CommandAction, Pointer<Parser>)>, ResponseError> {
    let mut msg = Message::default();
    updated_msg.apply_to_message(&mut msg);
    handle_command(ctx, msg).await
}

pub async fn handle_command(
    ctx: &Context,
    msg: Message,
) -> Result<Option<(String, CommandAction, Pointer<Parser>)>, ResponseError> {
    if msg.author.bot {
        return Ok(None);
    }

    let Some(command_manager) = CommandManager::extract_context(ctx).await else {
        error!("CommandManager not found in TypeMap");
        return Err(ResponseError::Err(
            "CommandManager not found in TypeMap".into(),
        ));
    };

    command_manager.execute(ctx, msg).await
}
