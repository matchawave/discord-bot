mod builder;
mod callback_functions;
mod functions;
pub mod response;
pub use builder::{CommandBuilder, ICommand};
pub use functions::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use serenity::{
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

    pub async fn register(&self, http: &HttpType, should_delete: bool) {
        let timer = ElapsedTime::new();
        let mut commands: Vec<CreateCommand> = Vec::new();
        for command in self.0.values() {
            let result: Result<Vec<CreateCommand>, _> = command.try_into();
            if let Ok(create_cmds) = result {
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
    async fn execute(
        &self,
        ctx: &Context,
        act: T,
    ) -> Result<Option<(CommandEvent, Pointer<Parser>)>, ResponseError>;
}

#[async_trait]
impl CommandExecution<Message> for CommandManager {
    async fn execute(
        &self,
        ctx: &Context,
        msg: Message,
    ) -> Result<Option<(CommandEvent, Pointer<Parser>)>, ResponseError> {
        let user_id = msg.author.id;
        let Some(guild_id) = msg.guild_id else {
            return Err(ResponseError::Err(format!(
                "Command in DMs: User {user_id}"
            )));
        };

        let pre_action: CommandAction = (&msg).into();
        let pre_event = CommandEvent {
            name: "name_not_parsed".into(),
            action: pre_action.clone(),
        };

        let prefix_option = match Prefix::extract_context_event(ctx, &pre_event).await {
            Some(Prefix(p)) => p.make_clone().await,
            None => None,
        };

        let prefix = match prefix_option {
            Some(p) => p,
            None => {
                let DefaultPrefix(dp) = DefaultPrefix::extract_context(ctx)
                    .await
                    .unwrap_or(DefaultPrefix("!".into()));
                dp
            }
        };

        if !msg.content.starts_with(&prefix) {
            return Ok(None);
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

        let Some(guild_map) = GuildMap::extract_context_event(ctx, &pre_event).await else {
            return Err(ResponseError::Err(format!(
                "Failed to extract GuildMap for guild {guild_id} for command execution"
            )));
        };

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

        let action: CommandAction = (&new_msg).into();
        let event = CommandEvent {
            name: command_name.clone(),
            action: action.clone(),
        };

        // * Get the command from the CommandManager
        let Some(command) = self.0.get(&command_name) else {
            return Err(ResponseError::Warn(format!(
                "Command '{}' not found for user {user_id} in guild {guild_id}",
                command_name
            )));
        };

        let command_prefix_print = format!("Command '{command_name}':");

        // Get the member who sent the command
        let Some(member) = (match Members::extract_context_event(ctx, &event).await {
            Some(members) => members.fetch(&ctx.http, (guild_id, user_id)).await,
            None => None,
        }) else {
            return Err(ResponseError::Err(format!(
                "{command_prefix_print} Failed to extract Member {user_id} in guild {guild_id}",
            )));
        };

        // Get the guild
        let guild = match Pointer::<PartialGuild>::extract_context_event(ctx, &event).await {
            Some(guild_ptr) => guild_ptr.make_clone().await,
            None => {
                return Err(ResponseError::Err(format!(
                    "{command_prefix_print} Failed to extract Guild {guild_id}"
                )));
            }
        };

        // ProcessManager for handling cooldowns and ephemerals
        let Some(p_manager) = Arc::<ProcessManager>::extract_context(ctx).await else {
            return Err(ResponseError::Err(format!(
                "{command_prefix_print} Failed to extract ProcessManager from context"
            )));
        };

        if guild.owner_id != member.user.id
            && let Some(fake_perms) = FakePerms::extract_context_event(ctx, &event).await
            && let Some(missing_perms) = fake_perms
                .member_lacks_permission(&member, command.permissions())
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

            let response = ResponseError::Warn(format!(
                "{}: You're **missing** {text}{missing_perms_str}",
                user_id.mention(),
            ));
            let response = create_error_embed(response);

            send_message(&ctx.http, &p_manager, &response, &msg);
            return Err(ResponseError::Warn(format!(
                "{command_prefix_print} User {user_id} is missing permissions in guild {guild_id}: {missing_perms_str}"
            )));
        }

        if let Some(cooldown_num) = command.cooldown()
            && let Some(cooldowns) = p_manager.get::<Cooldowns>()
        {
            let cooldown = Cooldown::Command(guild_id, user_id, command_name.clone());
            if (cooldowns.0.read().await).get(&cooldown).is_some() {
                return Err(ResponseError::Warn(format!(
                    "{command_prefix_print} cooldown for user {user_id} in guild {guild_id}"
                )));
            } else {
                (cooldowns.0.write().await).insert(
                    cooldown,
                    Instant::now() + Duration::from_millis(cooldown_num),
                );
            }
        }
        let parser = Pointer::new(Parser::new(ctx.shard_id));
        if let Some(func) = command.legacy()
            && let Some(response) = func.call(ctx, &event, &parser).await
        {
            send_message(&ctx.http, &p_manager, &response, &msg);
        }

        Ok(Some((event, parser.clone())))
    }
}

#[async_trait]
impl CommandExecution<CommandInteraction> for CommandManager {
    async fn execute(
        &self,
        ctx: &Context,
        interaction: CommandInteraction,
    ) -> Result<Option<(CommandEvent, Pointer<Parser>)>, ResponseError> {
        let Some(guild_id) = interaction.guild_id else {
            return Err(ResponseError::Err(format!(
                "Command in DMs: User {}",
                interaction.user.id
            )));
        };

        let command_name = interaction.data.name.to_lowercase();

        let Some(command) = self.0.get(&command_name) else {
            return Err(ResponseError::Warn(format!(
                "Command '{}' not found for user {} in guild {guild_id}",
                command_name, interaction.user.id
            )));
        };
        let mut parser = Parser::new(ctx.shard_id);
        let action = CommandAction::from(&interaction);
        let event = CommandEvent {
            name: command_name.clone(),
            action: action.clone(),
        };

        if let Some(member) = &interaction.member
            && let Some(Members(members)) = Members::extract_context_event(ctx, &event).await
        {
            let user_id = member.user.id;
            let member = *member.clone();
            parser.with_member(member.clone());
            if !members.contains_key(&user_id) {
                members.insert(user_id, Pointer::new(member)).await;
            }
        }

        if let Some(guild) = Pointer::<PartialGuild>::extract_context_event(ctx, &event).await {
            parser.with_guild(guild.make_clone().await);
        }

        if let Some(channels) = Channels::extract_context_event(ctx, &event).await
            && let Some(channel) = channels
                .fetch(&ctx.http, (guild_id, interaction.channel_id))
                .await
        {
            parser.with_channel(None, channel); // Get category from channel later if needed
        }

        let parser = Pointer::new(parser);

        if let Some(func) = command.slash()
            && let Some(response) = func.call(ctx, &event, &parser).await
        {
            let response_msg: CreateInteractionResponse = match (&response).try_into() {
                Ok(m) => m,
                Err(e) => {
                    return Err(ResponseError::Err(format!(
                        "{command_name}: Failed to create interaction response message:\n{e}",
                    )));
                }
            };

            let attachments = response.get_attachments().clone();

            tokio::spawn({
                let http = ctx.http.clone();
                let token = interaction.token.clone();
                let command_name = command_name.clone();
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
                        error!("{command_name}: interaction response failed:\n{}", e);
                    }
                }
            });
        }
        Ok(Some((event, parser.clone())))
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
