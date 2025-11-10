use serenity::{
    all::{
        CreateActionRow, CreateAllowedMentions, CreateAttachment, CreateEmbed,
        CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreatePoll,
        create_poll::Ready,
    },
    constants,
};
use utils::{ResponseError, command_error};

const MAX_COMPONENTS: usize = 10;
const MAX_EMBEDS: usize = 10;
const MAX_ATTACHMENTS: usize = 10;

#[derive(Debug, Default)]
pub struct CommandResponse {
    content: Option<String>,
    embeds: Vec<CreateEmbed>,
    components: Vec<CreateActionRow>,
    attachments: Vec<CreateAttachment>,
    poll: Option<CreatePoll<Ready>>,
    ephemeral: bool,
    reply: bool,
    mention: bool,
}

impl CommandResponse {
    pub fn new(content: impl Into<String>) -> Self {
        let content = content.into();
        let content = if content.is_empty() {
            None
        } else {
            Some(content)
        };
        Self {
            content,
            ..Default::default()
        }
    }

    pub fn new_embeds(embeds: Vec<CreateEmbed>) -> Self {
        Self {
            embeds,
            ..Default::default()
        }
    }

    pub fn new_components(components: Vec<CreateActionRow>) -> Self {
        Self {
            components,
            ..Default::default()
        }
    }

    pub fn new_attachments(attachments: Vec<CreateAttachment>) -> Self {
        Self {
            attachments,
            ..Default::default()
        }
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        let content = content.into();
        self.content = Some(content);
        self
    }

    pub fn embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds = embeds;
        self
    }

    pub fn components(mut self, components: Vec<CreateActionRow>) -> Self {
        self.components = components;
        self
    }

    pub fn add_attachment(mut self, attachment: CreateAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    pub fn get_attachments(&self) -> &Vec<CreateAttachment> {
        &self.attachments
    }

    pub fn add_embed(mut self, embed: CreateEmbed) -> Self {
        self.embeds.push(embed);
        self
    }

    pub fn poll(mut self, poll: CreatePoll<Ready>) -> Self {
        self.poll = Some(poll);
        self
    }

    pub fn ephemeral(mut self) -> Self {
        self.ephemeral = true;
        self
    }

    pub fn reply(mut self) -> Self {
        self.reply = true;
        self
    }

    pub fn mention(mut self) -> Self {
        self.mention = true;
        self
    }

    pub fn should_reply(&self) -> bool {
        self.reply
    }

    pub fn is_ephemeral(&self) -> bool {
        self.ephemeral
    }
}

impl TryInto<CreateMessage> for &CommandResponse {
    type Error = ResponseError;
    fn try_into(self) -> Result<CreateMessage, Self::Error> {
        let mut msg = CreateMessage::new();
        if let Some(ref content) = self.content {
            if content.len() > constants::MESSAGE_CODE_LIMIT {
                return command_error!(
                    "Content length exceeds limit: {} (max {})",
                    content.len(),
                    constants::MESSAGE_CODE_LIMIT
                );
            }
            if !content.is_empty() {
                msg = msg.content(content);
            }
        }
        if !self.embeds.is_empty() {
            if self.embeds.len() > constants::EMBED_MAX_COUNT {
                return command_error!(
                    "Too many embeds: {} (max {})",
                    self.embeds.len(),
                    constants::EMBED_MAX_COUNT
                );
            }
            msg = msg.embeds(self.embeds.clone());
        }
        if !self.components.is_empty() {
            if self.components.len() > MAX_COMPONENTS {
                return command_error!(
                    "Too many components: {} (max {})",
                    self.components.len(),
                    MAX_COMPONENTS
                );
            }
            msg = msg.components(self.components.clone());
        }
        if !self.attachments.is_empty() {
            msg = msg.add_files(self.attachments.clone());
        }
        if let Some(ref poll) = self.poll {
            msg = msg.poll(poll.clone());
        }

        Ok(msg)
    }
}

impl TryInto<CreateInteractionResponse> for &CommandResponse {
    type Error = ResponseError;

    fn try_into(self) -> Result<CreateInteractionResponse, Self::Error> {
        let mut msg = CreateInteractionResponseMessage::new();

        if let Some(ref content) = self.content {
            if content.len() > constants::MESSAGE_CODE_LIMIT {
                return command_error!(
                    "Content length exceeds limit: {} (max {})",
                    content.len(),
                    constants::MESSAGE_CODE_LIMIT
                );
            }
            msg = msg.content(content);
        }
        if !self.embeds.is_empty() {
            if self.embeds.len() > constants::EMBED_MAX_COUNT {
                return command_error!(
                    "Too many embeds: {} (max {})",
                    self.embeds.len(),
                    constants::EMBED_MAX_COUNT
                );
            }
            msg = msg.embeds(self.embeds.clone());
        }
        if !self.components.is_empty() {
            if self.components.len() > MAX_COMPONENTS {
                return command_error!(
                    "Too many components: {} (max {})",
                    self.components.len(),
                    MAX_COMPONENTS
                );
            }
            msg = msg.components(self.components.clone());
        }
        if !self.attachments.is_empty() {
            msg = msg.add_files(self.attachments.clone());
        }
        if let Some(ref poll) = self.poll {
            msg = msg.poll(poll.clone());
        }

        msg = msg.ephemeral(self.ephemeral);
        Ok(CreateInteractionResponse::Message(msg))
    }
}

impl From<&CommandResponse> for CreateAllowedMentions {
    /// Set allowed mentions based on whether to mention the replied user
    fn from(response: &CommandResponse) -> Self {
        CreateAllowedMentions::new().replied_user(response.mention)
    }
}

impl<T: Into<String>> From<T> for CommandResponse {
    fn from(content: T) -> Self {
        CommandResponse::new(content.into())
    }
}
