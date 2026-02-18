use serenity::all::{CreateActionRow, CreateEmbed};

use crate::{
    command::{CommandResult, create_error_embed, response::CommandResponse},
    handler::CallbackReturn,
};

impl CallbackReturn<CommandResponse> for CommandResponse {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        Some(*self)
    }
}

impl CallbackReturn<CommandResponse> for Option<CommandResponse> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        *self
    }
}

impl CallbackReturn<CommandResponse> for CommandResult<CommandResponse> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(response) => response,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}

impl CallbackReturn<CommandResponse> for CommandResult<CreateEmbed> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(Some(embed)) => Some(CommandResponse::new_embeds(vec![embed]).reply()),
            Ok(None) => None,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}

impl CallbackReturn<CommandResponse> for CommandResult<Vec<CreateEmbed>> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(Some(embeds)) => Some(CommandResponse::new_embeds(embeds).reply()),
            Ok(None) => None,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}

impl CallbackReturn<CommandResponse> for CommandResult<CreateActionRow> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(Some(components)) => Some(CommandResponse::new_components(vec![components]).reply()),
            Ok(None) => None,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}
impl CallbackReturn<CommandResponse> for CommandResult<Vec<CreateActionRow>> {
    fn into_response(self: Box<Self>) -> Option<CommandResponse> {
        match *self {
            Ok(Some(components)) => Some(CommandResponse::new_components(components).reply()),
            Ok(None) => None,
            Err(e) => Some(create_error_embed(e)),
        }
    }
}
impl CallbackReturn<CommandResponse> for CommandResult<(Vec<CreateEmbed>, Vec<CreateActionRow>)> {
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
