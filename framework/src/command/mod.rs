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

use utils::{ElapsedTime, HttpType, Parser, Pointer, ResponseError, debug, error, info};

use crate::{
    command::response::CommandResponse,
    data::{CommandAliases, Cooldown, Cooldowns, DefaultPrefix, Ephemeral, Ephemerals, Prefix},
    extractors::{ContextEventExtractor, ContextExtractor},
    global::Commands,
    guilds::{Channels, FakePerms, GuildMap, HTTPGetter, Members},
    processes::ProcessManager,
};

pub type CommandResult<T = CommandResponse> = Result<Option<T>, ResponseError>;

#[derive(Default, Clone)]
pub struct CommandManagerBuilder(Vec<ICommand>);

impl CommandManagerBuilder {
    pub fn add_command(mut self, command: ICommand) -> Self {
        self.0.push(command);
        self
    }

    pub fn add_commands(mut self, commands: Vec<ICommand>) -> Self {
        self.0.extend(commands);
        self
    }

    pub fn build(self) -> CommandManager {
        let mut map = HashMap::new();
        for c in self.0.iter() {
            let name = c.name().to_string();
            map.insert(name, c.clone());
        }
        CommandManager(map.into())
    }
}

#[derive(Default, Clone)]
pub struct CommandManager(Arc<HashMap<String, ICommand>>);

impl CommandManager {
    pub fn set(&self, data: &mut TypeMap) {
        data.insert::<Commands>(self.clone());
    }

    pub async fn register(&self, client: &Client, should_delete: bool) {
        let http = &client.http;
        let timer = ElapsedTime::new();
        let mut commands: Vec<CreateCommand> = Vec::new();
        for cmd in self.0.values() {
            let res: Result<Vec<CreateCommand>, _> = cmd.try_into();
            if let Ok(create_cmds) = res {
                commands.extend(create_cmds);
            }
        }

        let dev_guild = GuildId::from(851102546470371338);

        if should_delete {
            let empty_vec: Vec<CreateCommand> = Vec::new();
            debug!("Deleting existing commands...");
            if let Err(e) = http.create_guild_commands(dev_guild, &empty_vec).await {
                error!("Failed to delete existing commands: {}", e);
            }
            // debug!("Deleting global commands...");
            // if let Err(e) = http.create_global_commands(&empty_vec).await {
            //     error!("Failed to delete global commands: {}", e);
            // }
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
impl ContextExtractor for CommandManager {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        let data_read = ctx.data.read().await;
        data_read.get::<Commands>().cloned()
    }
}

#[async_trait]
pub(crate) trait CommandExecution<T> {
    async fn execute(&self, ctx: &Context, act: T) -> Option<String>;
}

#[async_trait]
impl CommandExecution<Message> for CommandManager {
    async fn execute(&self, ctx: &Context, msg: Message) -> Option<String> {
        let guild_id = msg.guild_id?;
        let pre_action: CommandAction = CommandAction::from(&msg);

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

        if !msg.content.starts_with(&prefix) {
            return None;
        }

        let (mut command_name, mut content) = {
            let content: String = msg.content.clone();
            let content = content.trim_start_matches(&prefix);
            let command_name = content
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_lowercase();
            let content = (content.trim_start_matches(&command_name).trim_start()).to_string();
            (command_name, content)
        };

        let guild_map = GuildMap::extract_context_event(ctx, &pre_action).await?;
        if let Some(command_aliases) = (guild_map.read().await)
            .get::<CommandAliases>()
            .map(CommandAliases::from)
            && let Some(alias) = command_aliases.get_cloned(&command_name).await
        {
            content = alias.args_as_string().unwrap_or(content);
            command_name = alias.command_name.clone();
        }

        let new_msg = {
            let mut new_msg = msg.clone();
            new_msg.content = content;
            new_msg
        };

        let action = CommandAction::from(&new_msg);

        let command = self.0.get(&command_name)?;

        let callbacks = &command.callbacks;

        // Get the member who sent the command
        let Some(member) = (match Members::extract_context_event(ctx, &action).await {
            Some(members) => members.fetch(&ctx.http, (guild_id, msg.author.id)).await,
            None => None,
        }) else {
            error!(
                "Failed to extract Member {} in guild {} for command '{}'",
                msg.author.id, guild_id, command_name
            );
            return None;
        };

        // Get the guild
        let guild = match Pointer::<PartialGuild>::extract_context_event(ctx, &action).await {
            Some(guild_ptr) => guild_ptr.make_clone().await,
            None => {
                error!(
                    "Failed to extract Guild {} for command '{}'",
                    guild_id, command_name
                );
                return None;
            }
        };

        // ProcessManager for handling cooldowns and ephemerals
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

            send_message(&ctx.http, &p_manager, &response, &msg);
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
        let parser = Pointer::new(Parser::new(ctx.shard_id));
        for callback in callbacks.iter() {
            if let CommandCallbackType::Legacy(func) = callback
                && let Some(response) = func.call(ctx, &action, &parser).await
            {
                send_message(&ctx.http, &p_manager, &response, &msg);
            };
        }
        Some(command_name)
    }
}

#[async_trait]
impl CommandExecution<CommandInteraction> for CommandManager {
    async fn execute(&self, ctx: &Context, interaction: CommandInteraction) -> Option<String> {
        let guild_id = interaction.guild_id?; // Ensure it's in a guild

        let c_name = interaction.data.name.to_lowercase();
        let action = CommandAction::from(&interaction);

        let command = self.0.get(&c_name)?;
        let mut parser = Parser::new(ctx.shard_id);

        if let Some(member) = &interaction.member
            && let Some(Members(members)) = Members::extract_context_event(ctx, &action).await
        {
            let user_id = member.user.id;
            let member = *member.clone();
            parser.with_member(member.clone());
            if !members.contains_key(&user_id) {
                members.insert(user_id, Pointer::new(member)).await;
            }
        }

        if let Some(guild) = Pointer::<PartialGuild>::extract_context_event(ctx, &action).await {
            parser.with_guild(guild.make_clone().await);
        }

        if let Some(channels) = Channels::extract_context_event(ctx, &action).await
            && let Some(channel) = channels
                .fetch(&ctx.http, (guild_id, interaction.channel_id))
                .await
        {
            parser.with_channel(None, channel); // Get category from channel later if needed
        }

        let callbacks = &command.callbacks;
        let parser = Pointer::new(parser);
        for callback in callbacks.iter() {
            if let CommandCallbackType::Slash(func) = callback
                && let Some(response) = func.call(ctx, &action, &parser).await
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
                    async move {
                        if let Err(e) = http
                            .create_interaction_response(
                                interaction.id,
                                &token,
                                &response_msg,
                                attachments,
                            )
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
    http: &HttpType,
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
