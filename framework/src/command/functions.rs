use std::sync::Arc;

use serenity::{
    all::{CommandInteraction, Context, Message},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::{
    HandlerFn,
    command::response::CommandResponse,
    extractors::{DynHandler, ExtractorTuple, HandlerBuilder},
};

pub trait CallbackReturn: Send + Sync {
    fn into_response(self: Box<Self>) -> Option<CommandResponse>;
}

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

#[async_trait]
pub trait DynCommandCallback: Send + Sync {
    async fn call(
        &self,
        ctx: &Context,
        action: &CommandAction,
        p: &Pointer<Parser>,
    ) -> Option<CommandResponse>;
}

#[async_trait]
impl<D> DynCommandCallback for D
where
    D: DynHandler<CommandAction> + 'static,
    D::Output: CallbackReturn,
{
    async fn call(
        &self,
        ctx: &Context,
        action: &CommandAction,
        p: &Pointer<Parser>,
    ) -> Option<CommandResponse> {
        if let Some(result) = DynHandler::call(self, ctx, action, p).await {
            return Box::new(result).into_response();
        }
        None
    }
}

type CommandCallback = Arc<dyn DynCommandCallback>;

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
        U: CallbackReturn + 'static,
    {
        let handler = HandlerBuilder::<CommandAction, U>::build(func);
        CommandCallbackType::Slash(Arc::new(handler))
    }

    pub fn legacy<F, U, Args>(func: F) -> Self
    where
        F: HandlerFn<Args, U> + Send + Sync + Copy + 'static,
        Args: ExtractorTuple<CommandAction> + Send + Sync + 'static,
        U: CallbackReturn + 'static,
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
