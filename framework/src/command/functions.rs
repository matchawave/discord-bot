use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serenity::{
    all::{CommandInteraction, Context, Message},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::{
    command::response::CommandResponse,
    extractors::{EventExtractor, Extractor, ExtractorTuple},
    handler::{CallbackReturn, DynCallback, DynHandler, HandlerBuilder, HandlerFn},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandAction {
    // Name of the command, and the event that triggered it
    Interaction(Box<CommandInteraction>),
    Message(Box<Message>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEvent {
    pub name: String,
    pub action: CommandAction,
}

impl From<&Message> for CommandAction {
    fn from(message: &Message) -> Self {
        CommandAction::Message(Box::new(message.clone()))
    }
}

impl From<&CommandInteraction> for CommandAction {
    fn from(interaction: &CommandInteraction) -> Self {
        CommandAction::Interaction(Box::new(interaction.clone()))
    }
}

type CommandCallback = Arc<dyn DynCallback<CommandAction, CommandResponse>>;

#[derive(Clone)]
pub enum CommandCallbackType {
    Slash(CommandCallback),
    Legacy(CommandCallback),
    Autocomplete(Arc<dyn DynHandler<CommandAction, Output = Vec<String>>>),
    User(CommandCallback),
    Message(CommandCallback),
}
impl CommandCallbackType {
    pub fn slash<F, U, Args>(func: F) -> Self
    where
        F: HandlerFn<Args, U> + Send + Sync + Copy + 'static,
        Args: ExtractorTuple<CommandAction> + Send + Sync + 'static,
        U: CallbackReturn<CommandResponse> + 'static,
    {
        let handler = HandlerBuilder::<CommandAction, U>::build(func);
        CommandCallbackType::Slash(Arc::new(handler))
    }

    pub fn legacy<F, U, Args>(func: F) -> Self
    where
        F: HandlerFn<Args, U> + Send + Sync + Copy + 'static,
        Args: ExtractorTuple<CommandAction> + Send + Sync + 'static,
        U: CallbackReturn<CommandResponse> + 'static,
    {
        let handler = HandlerBuilder::<CommandAction, U>::build(func);
        CommandCallbackType::Legacy(Arc::new(handler))
    }

    pub fn autocomplete<F, Args>(func: F) -> Self
    where
        F: HandlerFn<Args, Vec<String>> + Send + Sync + Copy + 'static,
        Args: ExtractorTuple<CommandAction> + Send + Sync + 'static,
    {
        let handler = HandlerBuilder::<CommandAction, Vec<String>>::build(func);
        CommandCallbackType::Autocomplete(Arc::new(handler))
    }
}

pub struct CommandName(pub String);

#[async_trait]
impl EventExtractor<CommandEvent> for CommandName {
    async fn extract_event(event: &CommandEvent) -> Option<Self> {
        Some(CommandName(event.name.clone()))
    }
}

#[async_trait]
impl Extractor<CommandEvent> for CommandName {
    async fn extract(_ctx: &Context, event: &CommandEvent, _p: &Pointer<Parser>) -> Option<Self> {
        Self::extract_event(event).await
    }
}
