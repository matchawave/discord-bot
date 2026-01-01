use serenity::all::{Context, Message, MessageUpdateEvent};
use utils::error;

use crate::{
    command::{CommandExecution, CommandManager},
    extractors::ContextExtractor,
};

pub async fn handle_edited_command(
    ctx: &Context,
    updated_msg: &MessageUpdateEvent,
) -> Option<String> {
    let mut msg = Message::default();
    updated_msg.apply_to_message(&mut msg);
    handle_command(ctx, msg).await
}

pub async fn handle_command(ctx: &Context, msg: Message) -> Option<String> {
    if msg.author.bot {
        return None;
    }

    let Some(command_manager) = CommandManager::extract_context(ctx).await else {
        error!("CommandManager not found in TypeMap");
        return None;
    };

    command_manager.execute(ctx, msg).await
}
