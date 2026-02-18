use std::sync::Arc;

use serenity::{
    all::{CommandInteraction, Message},
    async_trait,
};

use crate::{
    command::response::CommandResponse,
    extractors::ExtractorTuple,
    handler::{CallbackReturn, DynCallback, DynHandler, HandlerBuilder, HandlerFn},
};

#[derive(Clone)]
pub enum CommandAction {
    Interaction(Box<CommandInteraction>),
    Message(Box<Message>),
}

impl From<&Message> for CommandAction {
    fn from(msg: &Message) -> Self {
        CommandAction::Message(Box::new(msg.clone()))
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
