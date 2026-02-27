use serde::{Deserialize, Serialize};
use serenity::{
    all::{CommandInteraction, Context, Message},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::extractors::{EventExtractor, Extractor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandAction {
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

impl From<&Message> for CommandEvent {
    fn from(message: &Message) -> Self {
        CommandEvent {
            name: message.content.clone(), // You might want to parse the command name properly
            action: CommandAction::from(message),
        }
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
