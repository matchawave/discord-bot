use serenity::all::{Colour, CreateActionRow, CreateEmbed};
use utils::ResponseError;

use crate::command::{CommandResult, functions::CallbackReturn, response::CommandResponse};

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

impl CallbackReturn for CommandResult<CommandResponse> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(response) => response,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}

impl CallbackReturn for CommandResult<CreateEmbed> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(Some(embed)) => Some(CommandResponse::new_embeds(vec![embed]).reply()),
            Ok(None) => None,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}

impl CallbackReturn for CommandResult<Vec<CreateEmbed>> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(Some(embeds)) => Some(CommandResponse::new_embeds(embeds).reply()),
            Ok(None) => None,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}

impl CallbackReturn for CommandResult<CreateActionRow> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(Some(components)) => Some(CommandResponse::new_components(vec![components]).reply()),
            Ok(None) => None,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}
impl CallbackReturn for CommandResult<Vec<CreateActionRow>> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(Some(components)) => Some(CommandResponse::new_components(components).reply()),
            Ok(None) => None,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}
impl CallbackReturn for CommandResult<(Vec<CreateEmbed>, Vec<CreateActionRow>)> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(Some((embeds, components))) => Some(
                CommandResponse::new_embeds(embeds)
                    .components(components)
                    .reply(),
            ),
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
