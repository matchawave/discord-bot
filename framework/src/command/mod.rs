mod builder;
mod callback_functions;
mod functions;
pub mod response;
pub use builder::ICommand;
pub use functions::{CommandAction, CommandCallbackType};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use serenity::{
    Client,
    all::{
        Colour, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
        CreateMessage, GuildId, Mentionable, Message, PartialGuild,
    },
    async_trait,
    prelude::TypeMap,
};
use tokio::sync::RwLock;
use utils::{ElapsedTime, Http, Parser, Pointer, ResponseError, debug, error, info};

use crate::{
    command::response::CommandResponse,
    data::{CommandAliases, Cooldown, Cooldowns, DefaultPrefix, Ephemeral, Ephemerals, Prefix},
    extractors::{ContextEventExtractor, ContextExtractor, Extractor},
    global::Commands,
    guilds::{FakePerms, GuildMap, HTTPGetter, Members},
    processes::ProcessManager,
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
        let pre_action: CommandAction = CommandAction::from(msg);

        let prefix_option = match Prefix::extract_context_event(ctx, &pre_action).await {
            Some(Prefix(p)) => p.make_clone().await,
            None => None,
        };

        let prefix = match prefix_option {
            Some(p) => p,
            None => {
                let DefaultPrefix(dp) = DefaultPrefix::extract_context(ctx)
                    .await
                    .unwrap_or(DefaultPrefix("!".to_string()));
                dp
            }
        };

        println!("Using prefix: {}", prefix);

        if !msg.content.starts_with(&prefix) {
            return None;
        }

        let content: String = msg.content.clone();
        let (mut command_name, content) = {
            let content = content.trim_start_matches(&prefix);
            let command_name = content
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_lowercase();
            let content = content.trim_start_matches(&command_name).trim_start();
            (command_name, content)
        };
        let guild_map = GuildMap::extract(ctx, &pre_action, parser).await?;
        let mut new_msg = msg.clone();
        new_msg.content = content.to_string();
        let mut action = CommandAction::from(&new_msg);

        if let Some(command_aliases) = (guild_map.read().await)
            .get::<CommandAliases>()
            .map(CommandAliases::from)
            && let Some(alias) = command_aliases.get_cloned(&command_name).await
        {
            new_msg.content = alias.args_as_string();
            action = CommandAction::from(&new_msg);
            command_name = alias.command_name.clone();
        }

        let command = self.0.read().await;
        let command = command.get(&command_name)?;

        let callbacks = &command.callbacks;

        let Some(member) = (match Members::extract(ctx, &action, parser).await {
            Some(members) => members.fetch(&ctx.http, (guild_id, msg.author.id)).await,
            None => None,
        }) else {
            error!(
                "Failed to extract Member {} in guild {} for command '{}'",
                msg.author.id, guild_id, command_name
            );
            return None;
        };

        let guild = match guild_map.read().await.get::<Pointer<PartialGuild>>() {
            Some(guild_ptr) => guild_ptr.make_clone().await,
            None => {
                error!(
                    "Failed to extract Guild {} for command '{}'",
                    guild_id, command_name
                );
                return None;
            }
        };

        let Some(p_manager) = Arc::<ProcessManager>::extract_context(ctx).await else {
            error!("Failed to extract ProcessManager from context");
            return None;
        };

        if guild.owner_id != member.user.id
            && let Some(fake_perms) = FakePerms::extract_context_event(ctx, &action).await
            && let Some(missing_perms) = fake_perms
                .member_lacks_permission(&member, &command.permissions)
                .await
        {
            let text = if missing_perms.len() == 1 {
                "permission: "
            } else {
                "permissions:\n"
            };

            let missing_perms_str = missing_perms
                .par_iter()
                .map(|p| format!("`{}`", p))
                .collect::<Vec<String>>()
                .join(", ");

            let response = create_error_embed(ResponseError::Warn(format!(
                "{}: You're **missing** {}{}",
                member.user.id.mention(),
                text,
                missing_perms_str
            )));

            send_message(&ctx.http, &p_manager, &response, msg);
            return Some(command_name);
        }

        if let Some(cooldown_num) = command.cooldown
            && let Some(cooldowns) = p_manager.get::<Cooldowns>()
        {
            let cooldown = Cooldown::Command(guild_id, msg.author.id, command_name.clone());
            if (cooldowns.0.read().await).get(&cooldown).is_some() {
                debug!(
                    "Command '{}' is on cooldown for user {} in guild {}",
                    command_name, msg.author.id, guild_id
                );
                return Some(command_name);
            } else {
                (cooldowns.0.write().await).insert(
                    cooldown,
                    Instant::now() + Duration::from_millis(cooldown_num),
                );
            }
        }

        for callback in callbacks.iter() {
            if let CommandCallbackType::Legacy(func) = callback
                && let Some(response) = func.call(ctx, &action, parser).await
            {
                send_message(&ctx.http, &p_manager, &response, msg);
            };
        }
        Some(command_name)
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
        interaction.guild_id?; // Ensure it's in a guild

        let c_name = interaction.data.name.to_lowercase();
        let action = CommandAction::from(interaction);

        let command = self.0.read().await;
        let command = command.get(&c_name)?;

        if let Some(member) = &interaction.member
            && let Some(Members(members)) = Members::extract_context_event(ctx, &action).await
        {
            let user_id = member.user.id;
            if !members.contains_key(&user_id) {
                let member = *member.clone();
                members.insert(user_id, Pointer::new(member)).await;
            }
        }

        let callbacks = &command.callbacks;
        for callback in callbacks.iter() {
            if let CommandCallbackType::Slash(func) = callback
                && let Some(response) = func.call(ctx, &action, parser).await
            {
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
        }
        Some(c_name)
    }
}

pub(super) fn send_message(
    http: &Http,
    p_manager: &Arc<ProcessManager>,
    response: &CommandResponse,
    ref_msg: &Message,
) {
    let channel_id = response.get_channel().unwrap_or(ref_msg.channel_id);
    let mut response_msg: CreateMessage = match response.try_into() {
        Ok(m) => m,
        Err(e) => {
            // This should send the error msg to the channel
            error!("Failed to create message from command response: {}", e);
            return;
        }
    };

    let attachments = response.get_attachments().clone();

    if response.should_reply() {
        response_msg = response_msg.reference_message(ref_msg);
        response_msg = response_msg.allowed_mentions(response.into()); // Set allowed mentions based on response
    }
    let is_ephemeral = response.is_ephemeral();
    let http = http.clone();
    let p_manager = p_manager.clone();
    tokio::spawn({
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

            if is_ephemeral && let Some(ephemerals) = p_manager.get::<Ephemerals>() {
                let mut map = ephemerals.0.write().await;
                map.insert(Ephemeral::new(&m), Instant::now() + Duration::from_secs(3));
            }
        }
    });
}

pub(super) fn create_error_embed(error: ResponseError) -> CommandResponse {
    let mut embed = CreateEmbed::default();
    embed = embed.description(error.to_string());
    match error {
        ResponseError::Err(_) => embed = embed.color(Colour::RED),
        ResponseError::Warn(_) => embed = embed.color(Colour::GOLD),
        ResponseError::Info(_) => embed = embed.color(Colour::BLITZ_BLUE),
    };

    CommandResponse::new_embeds(vec![embed]).ephemeral()
}
