use std::sync::Arc;

use serenity::{
    all::{Colour, CommandInteraction, Context, CreateEmbed, Message},
    async_trait,
};
use utils::{Parser, Pointer, ResponseError};

use crate::{
    HandlerFn,
    command::response::CommandResponse,
    extractors::{DynHandler, ExtractorTuple, HandlerBuilder},
};

pub trait CallbackReturn: Send + Sync {
    fn into_response(self: Box<Self>) -> Option<CommandResponse>;
}

impl CallbackReturn for () {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        None
    }
}

impl CallbackReturn for CommandResponse {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        Some(*self)
    }
}

impl CallbackReturn for Option<CommandResponse> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        *self
    }
}

impl CallbackReturn for super::CommandResult<CommandResponse> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(response) => response,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}

impl CallbackReturn for super::CommandResult<CreateEmbed> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(Some(embed)) => Some(CommandResponse::new_embeds(vec![embed]).reply()),
            Ok(None) => None,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}

impl CallbackReturn for super::CommandResult<Vec<CreateEmbed>> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(Some(embeds)) => Some(CommandResponse::new_embeds(embeds).reply()),
            Ok(None) => None,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}

fn create_error_embed(error: ResponseError) -> CommandResponse {
    let mut embed = CreateEmbed::default();
    embed = embed.description(error.to_string());
    match error {
        ResponseError::Err(_) => embed = embed.color(Colour::RED),
        ResponseError::Warn(_) => embed = embed.color(Colour::GOLD),
        ResponseError::Info(_) => embed = embed.color(Colour::BLITZ_BLUE),
    };

    CommandResponse::new_embeds(vec![embed]).ephemeral()
}

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
