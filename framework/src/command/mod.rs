mod builder;
mod callback_functions;
mod functions;
pub mod response;
pub use builder::ICommand;
pub use functions::{CommandAction, CommandCallbackType};

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use serenity::{
    Client,
    all::{
        CommandInteraction, Context, CreateCommand, CreateInteractionResponse, CreateMessage,
        GuildId, Member, Message,
    },
    async_trait,
    prelude::TypeMap,
};
use tokio::sync::RwLock;
use utils::{ElapsedTime, Parser, Pointer, ResponseError, debug, error, info};

use crate::{
    Extractable,
    cache::Members,
    command::response::CommandResponse,
    data::{Data, Ephemeral, Ephemerals, Prefixes},
    extractors::Extractor,
    global::Commands,
};

pub type CommandResult<T = CommandResponse> = Result<Option<T>, ResponseError>;

#[derive(Default, Clone)]
pub struct CommandManager(Arc<Pointer<HashMap<String, ICommand>>>);

impl CommandManager {
    pub fn set(&self, data: &mut TypeMap) {
        data.insert::<Commands>(self.clone());
    }

    pub fn insert(&self, command: &ICommand) {
        let name = command.name();
        tokio::task::block_in_place(|| {
            self.0
                .write_sync()
                .insert(name.to_string(), command.clone());
        });
    }

    pub fn insert_all(&self, commands: Vec<ICommand>) {
        for c in commands.iter() {
            self.insert(c)
        }
    }

    pub(crate) async fn get(data: &Arc<RwLock<TypeMap>>) -> Option<CommandManager> {
        let data = data.read().await;
        match data.get::<Commands>() {
            Some(manager) => Some(manager.clone()),
            None => {
                error!("Failed to get CommandManager from TypeMap");
                None
            }
        }
    }

    pub async fn register(&self, client: &Client, should_delete: bool) {
        let http = &client.http;
        let timer = ElapsedTime::new();
        let commands: Vec<CreateCommand> = (self.0.read().await)
            .iter()
            .flat_map(|c| {
                let c: Vec<CreateCommand> = c.1.into();
                c
            })
            .collect();

        let dev_guild = GuildId::from(851102546470371338);

        if should_delete {
            let empty_vec: Vec<CreateCommand> = Vec::new();
            debug!("Deleting existing commands...");
            if let Err(e) = http.create_guild_commands(dev_guild, &empty_vec).await {
                error!("Failed to delete existing commands: {}", e);
            }
            debug!("Deleting global commands...");
            if let Err(e) = http.create_global_commands(&empty_vec).await {
                error!("Failed to delete global commands: {}", e);
            }
            debug!("Deleted All existing commands.");
        }

        if let Err(e) = http.create_guild_commands(dev_guild, &commands).await {
            panic!("Failed to register commands: {}", e);
        }

        info!(
            "Registered {} commands in {}ms",
            commands.len(),
            timer.elapsed_ms()
        );
    }
}

#[async_trait]
pub(crate) trait CommandExecution<T> {
    async fn execute(&self, ctx: &Context, act: &T, parser: &Pointer<Parser>) -> Option<String>;
}

#[async_trait]
impl CommandExecution<Message> for CommandManager {
    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        parser: &Pointer<Parser>,
    ) -> Option<String> {
        let guild_id = msg.guild_id?;

        let shard_data = Data::get(&ctx.data, ctx.shard_id).await?;
        let prefixes = Prefixes::retrieve(&shard_data)?;
        let prefix = prefixes.get(guild_id).await;

        let content = msg.content.clone();

        if !content.starts_with(&prefix) {
            return None;
        }

        let content = content.trim_start_matches(&prefix);
        let c_name = content
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_lowercase();

        let content = content.trim_start_matches(&c_name).trim_start();
        let command = self.0.read().await;
        let command = command.get(&c_name)?;

        let mut new_msg = msg.clone();
        new_msg.content = content.to_string();

        let action = CommandAction::from(&new_msg);

        if let Some(part_member) = &msg.member {
            let members = Members::extract(ctx, &action, parser).await?;
            let key = (guild_id, msg.author.id);
            if !members.contains(key).await {
                let mut member: Member = (*part_member.clone()).into();
                member.user = msg.author.clone();
                members.insert(key, member).await;
            }
        }

        let callbacks = &command.callbacks;
        // if let Some(cooldown) = command.cooldown {}
        for callback in callbacks.iter() {
            let CommandCallbackType::Legacy(func) = callback else {
                continue;
            };
            let Some(response) = func.call(ctx, &action, parser).await else {
                continue;
            };

            let mut response_msg: CreateMessage = match (&response).try_into() {
                Ok(m) => m,
                Err(e) => {
                    //This should send the error msg to the channel
                    error!("Failed to create message from command response: {}", e);
                    continue;
                }
            };

            let attachments = response.get_attachments().clone();

            if response.should_reply() {
                response_msg = response_msg.reference_message(msg);
                response_msg = response_msg.allowed_mentions((&response).into()); // Set allowed mentions based on response
            }

            let possible_channel_id = response.get_channel();

            tokio::spawn({
                let http = ctx.http.clone();
                let channel_id = possible_channel_id.unwrap_or(msg.channel_id);
                let shard_data = shard_data.clone();
                async move {
                    let m = match http
                        .send_message(channel_id, attachments.clone(), &response_msg)
                        .await
                    {
                        Ok(m) => m,
                        Err(e) => {
                            error!("Failed to send command response message: {}", e);
                            return;
                        }
                    };

                    if response.is_ephemeral()
                        && let Some(Ephemerals(ephemerals)) = Ephemerals::retrieve(&shard_data)
                    {
                        let mut map = ephemerals.write().await;
                        map.insert(Ephemeral::new(&m), Instant::now() + Duration::from_secs(3));
                    }
                }
            });
        }
        Some(c_name)
    }
}

#[async_trait]
impl CommandExecution<CommandInteraction> for CommandManager {
    async fn execute(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
        parser: &Pointer<Parser>,
    ) -> Option<String> {
        let _guild_id = interaction.guild_id?;

        let c_name = interaction.data.name.to_lowercase();
        let action = CommandAction::from(interaction);

        let command = self.0.read().await;
        let command = command.get(&c_name)?;

        if let Some(member) = &interaction.member {
            let members = Members::extract(ctx, &action, parser).await?;
            let key = (member.guild_id, member.user.id);
            if !members.contains(key).await {
                let member = *member.clone();
                members.insert(key, member).await;
            }
        }

        let callbacks = &command.callbacks;
        for callback in callbacks.iter() {
            let CommandCallbackType::Slash(func) = callback else {
                continue;
            };
            let Some(response) = func.call(ctx, &action, parser).await else {
                continue;
            };

            let response_msg: CreateInteractionResponse = match (&response).try_into() {
                Ok(m) => m,
                Err(e) => {
                    //This should send the error msg to the channel
                    error!("Failed to create message from command response: {}", e);
                    continue;
                }
            };

            let attachments = response.get_attachments().clone();

            tokio::spawn({
                let http = ctx.http.clone();
                let token = interaction.token.clone();
                let id = interaction.id;
                async move {
                    if let Err(e) = http
                        .create_interaction_response(id, &token, &response_msg, attachments)
                        .await
                    {
                        error!("Failed to send command interaction response message: {}", e);
                    }
                }
            });
        }
        Some(c_name)
    }
}
