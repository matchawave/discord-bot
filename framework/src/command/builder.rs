use super::functions::CommandCallbackType;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serenity::all::{CommandType, CreateCommand, CreateCommandOption};
use utils::{BotPermission, info};
pub type StoredCommand = (Vec<CommandCallbackType>, Vec<BotPermission>);

#[derive(Clone)]
pub struct ICommand {
    name: String,
    description: String,
    options: Vec<CreateCommandOption>,
    pub(crate) cooldown: Option<u64>,
    pub(crate) permissions: Vec<BotPermission>,
    pub(crate) callbacks: Vec<CommandCallbackType>,
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
            callbacks: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn permissions(mut self, permissions: Vec<BotPermission>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn cooldown(mut self, cooldown: u64) -> Self {
        self.cooldown = Some(cooldown);
        self
    }

    pub fn options(mut self, options: Vec<CreateCommandOption>) -> Self {
        self.options = options;
        self
    }

    pub fn add_callback(mut self, callback: CommandCallbackType) -> Self {
        self.callbacks.push(callback);
        self
    }

    pub fn callbacks(mut self, callback: Vec<CommandCallbackType>) -> Self {
        self.callbacks = callback;
        self
    }
}

impl Into<Vec<CreateCommand>> for &ICommand {
    fn into(self) -> Vec<CreateCommand> {
        if self.callbacks.is_empty() {
            return Vec::new();
        }
        let mut commands = Vec::new();

        let callbacks = &self.callbacks;
        let mut types = Vec::new();
        if callbacks
            .par_iter()
            .find_first(|c| matches!(c, CommandCallbackType::Slash(_)))
            .is_some()
        {
            let mut cmd = CreateCommand::new(self.name.clone())
                .kind(CommandType::ChatInput)
                .description(self.description.clone())
                .nsfw(false);
            for option in &self.options {
                cmd = cmd.add_option(option.clone());
            }

            commands.push(cmd);
            types.push("slash");
        }

        if callbacks
            .par_iter()
            .find_first(|c| matches!(c, CommandCallbackType::User(_)))
            .is_some()
        {
            let cmd = CreateCommand::new(self.name.clone())
                .kind(CommandType::User)
                .description(self.description.clone())
                .nsfw(false);

            commands.push(cmd);
            types.push("user");
        }

        if callbacks
            .par_iter()
            .find_first(|c| matches!(c, CommandCallbackType::Message(_)))
            .is_some()
        {
            let cmd = CreateCommand::new(self.name.clone())
                .kind(CommandType::Message)
                .description(self.description.clone())
                .nsfw(false);

            commands.push(cmd);
            types.push("message");
        }

        info!("Loaded '{}' command: [ {} ]", self.name, types.join(", "));
        commands
    }
}

impl From<&ICommand> for StoredCommand {
    fn from(command: &ICommand) -> Self {
        // For simplicity, we return the first callback and permission
        // In a real implementation, you might want to handle multiple callbacks and permissions
        let callbacks = command.callbacks.clone();
        let permissions = command.permissions.clone();
        (callbacks, permissions)
    }
}
