use std::sync::Arc;

use crate::{
    command::{CommandEvent, response::CommandResponse},
    extractors::ExtractorTuple,
    handler::{CallbackReturn, DynCallback, DynHandler, HandlerBuilder, HandlerFn},
};

use serenity::all::{CommandType, CreateCommand, CreateCommandOption};
use utils::{BotPermission, info};

#[derive(Clone)]
pub struct ICommand {
    name: String,
    description: String,
    options: Vec<CreateCommandOption>,
    cooldown: Option<u64>,
    permissions: Vec<BotPermission>,
    slash_callback: Option<CommandCallback<CommandResponse>>,
    legacy_callback: Option<CommandCallback<CommandResponse>>,
    autocomplete_callback: Option<AutocompleteCallback>,
    user_callback: Option<CommandCallback<CommandResponse>>,
    message_callback: Option<CommandCallback<CommandResponse>>,
}

#[macro_export]
macro_rules! command_callbacks {
    () => { Vec::new() };
    ( $( $callback:expr ),* ) => {
        {
            Vec::from([ $( $callback, )* ])
        }
    };
}

impl ICommand {
    pub fn new<S: Into<String>>(name: S, description: S) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            cooldown: None,
            options: Vec::new(),
            permissions: Vec::new(),
            slash_callback: None,
            legacy_callback: None,
            autocomplete_callback: None,
            user_callback: None,
            message_callback: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn permissions(&self) -> &Vec<BotPermission> {
        &self.permissions
    }

    pub fn cooldown(&self) -> Option<u64> {
        self.cooldown
    }

    pub fn options(&self) -> &Vec<CreateCommandOption> {
        &self.options
    }

    pub fn slash(&self) -> Option<CommandCallback<CommandResponse>> {
        self.slash_callback.clone()
    }

    pub fn legacy(&self) -> Option<CommandCallback<CommandResponse>> {
        self.legacy_callback.clone()
    }

    pub fn autocomplete(&self) -> Option<AutocompleteCallback> {
        self.autocomplete_callback.clone()
    }
}

impl TryInto<Vec<CreateCommand>> for &ICommand {
    type Error = String;

    fn try_into(self) -> Result<Vec<CreateCommand>, Self::Error> {
        let name = self.name();

        let mut commands = Vec::new();
        let mut types = Vec::new();

        if self.legacy_callback.is_some() {
            types.push("legacy");
        }

        if self.slash_callback.is_some() {
            let mut cmd = CreateCommand::new(self.name.clone())
                .kind(CommandType::ChatInput)
                .description(self.description.clone())
                .nsfw(false);
            for option in &self.options {
                cmd = cmd.add_option(option.clone());
            }

            commands.push(cmd);
            types.push("slash");

            if self.autocomplete_callback.is_some() {
                types.push("autocomplete");
            }
        }

        if self.user_callback.is_some() {
            let cmd = CreateCommand::new(self.name.clone())
                .kind(CommandType::User)
                .description(self.description.clone())
                .nsfw(false);

            commands.push(cmd);
            types.push("user");
        }

        if self.message_callback.is_some() {
            let cmd = CreateCommand::new(self.name.clone())
                .kind(CommandType::Message)
                .description(self.description.clone())
                .nsfw(false);

            commands.push(cmd);
            types.push("message");
        }

        if commands.is_empty() {
            return Err(format!(
                "No valid command types found for command '{}'",
                name
            ));
        }

        info!("Loaded '{}' command: [ {} ]", self.name, types.join(", "));
        Ok(commands)
    }
}

type CommandCallback<T> = Arc<dyn DynCallback<CommandEvent, T>>;
type AutocompleteCallback = Arc<dyn DynHandler<CommandEvent, Output = Vec<String>>>;

#[derive(Default)]
pub struct CommandBuilder {
    options: Vec<CreateCommandOption>,
    cooldown: Option<u64>,
    permissions: Vec<BotPermission>,
    slash_callback: Option<CommandCallback<CommandResponse>>,
    legacy_callback: Option<CommandCallback<CommandResponse>>,
    autocomplete_callback: Option<AutocompleteCallback>,
    user_callback: Option<CommandCallback<CommandResponse>>,
    message_callback: Option<CommandCallback<CommandResponse>>,
}

impl CommandBuilder {
    pub fn permissions(mut self, permissions: &[BotPermission]) -> Self {
        self.permissions = permissions.to_vec();
        self
    }

    pub fn cooldown(mut self, cooldown: u64) -> Self {
        self.cooldown = Some(cooldown);
        self
    }

    pub fn options(mut self, options: &[CreateCommandOption]) -> Self {
        self.options = options.to_vec();
        self
    }

    pub fn slash<F, U, Args>(mut self, func: F) -> Self
    where
        F: HandlerFn<Args, U> + Send + Sync + Copy + 'static,
        Args: ExtractorTuple<CommandEvent> + Send + Sync + 'static,
        U: CallbackReturn<CommandResponse> + 'static,
    {
        self.slash_callback = Some(Arc::new(HandlerBuilder::<CommandEvent, U>::build(func)));
        self
    }

    pub fn legacy<F, U, Args>(mut self, func: F) -> Self
    where
        F: HandlerFn<Args, U> + Send + Sync + Copy + 'static,
        Args: ExtractorTuple<CommandEvent> + Send + Sync + 'static,
        U: CallbackReturn<CommandResponse> + 'static,
    {
        self.legacy_callback = Some(Arc::new(HandlerBuilder::<CommandEvent, U>::build(func)));
        self
    }

    pub fn autocomplete<F, Args>(mut self, func: F) -> Self
    where
        F: HandlerFn<Args, Vec<String>> + Send + Sync + Copy + 'static,
        Args: ExtractorTuple<CommandEvent> + Send + Sync + 'static,
    {
        let handler = HandlerBuilder::<CommandEvent, Vec<String>>::build(func);
        self.autocomplete_callback = Some(Arc::new(handler));
        self
    }

    // Similar methods for user_callback and message_callback can be added here

    pub fn build<T: Into<String>>(self, name: T, description: T) -> ICommand {
        ICommand {
            name: name.into(),
            description: description.into(),
            options: self.options,
            cooldown: self.cooldown,
            permissions: self.permissions,
            slash_callback: self.slash_callback,
            legacy_callback: self.legacy_callback,
            autocomplete_callback: self.autocomplete_callback,
            user_callback: self.user_callback,
            message_callback: self.message_callback,
        }
    }
}
