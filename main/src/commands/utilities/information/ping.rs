use std::{sync::Arc, time::Duration};

use framework::command::{CommandCallbackType, CommandResult, ICommand};
use serenity::all::{
    CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse, EditMessage, Message, ShardId, ShardManager,
};
use utils::{ElapsedTime, Http, command_warn, error};

const NAME: &str = "ping";
const DESCRIPTION: &str = "Check the bot's latency";

pub fn command() -> ICommand {
    // let options = CreateCommandOption::new(CommandOptionType::String, "", "");
    ICommand::new(NAME, DESCRIPTION)
        .cooldown(3000)
        .callbacks(vec![
            CommandCallbackType::slash(interaction),
            CommandCallbackType::legacy(legacy),
        ])
}

const PING_RESPONSE_TEMPLATE: &str = "Pong! It took {time} to ping.";

async fn interaction(
    shard_id: ShardId,
    shard_manager: Arc<ShardManager>,
    http: Http,
    interaction: CommandInteraction,
) -> CommandResult {
    let Some(latency) = get_latency(shard_id, shard_manager).await else {
        return command_warn!("Pong! Latency information is unavailable.");
    };
    let mut response_message = String::from(PING_RESPONSE_TEMPLATE)
        .replace("{time}", &format!("{}ms", latency.as_millis()));

    let edit_timer = ElapsedTime::new();
    let response = CreateInteractionResponseMessage::default().content(response_message.clone());
    if let Err(e) = interaction
        .create_response(http.clone(), CreateInteractionResponse::Message(response))
        .await
    {
        error!("Failed to create ping response: {:?}", e);
    }
    response_message += format!(" (edit: {}ms)", edit_timer.elapsed_ms()).as_str();
    if let Err(e) = interaction
        .edit_response(
            http.clone(),
            EditInteractionResponse::new().content(response_message),
        )
        .await
    {
        error!("Failed to edit ping response: {:?}", e);
    }

    Ok(None)
}

async fn legacy(
    shard_id: ShardId,
    shard_manager: Arc<ShardManager>,
    http: Http,
    msg: Message,
) -> CommandResult {
    let Some(latency) = get_latency(shard_id, shard_manager).await else {
        return command_warn!("Pong! Latency information is unavailable.");
    };
    let mut response_message = String::from(PING_RESPONSE_TEMPLATE)
        .replace("{time}", &format!("{}ms", latency.as_millis()));

    let edit_timer = ElapsedTime::new();
    let mut sent_msg = match msg.reply(http.clone(), response_message.clone()).await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to send ping response: {:?}", e);
            return Ok(None);
        }
    };
    response_message += format!(" (edit: {}ms)", edit_timer.elapsed_ms()).as_str();
    if let Err(e) = sent_msg
        .edit(http.clone(), EditMessage::new().content(response_message))
        .await
    {
        error!("Failed to edit ping response: {:?}", e);
    }

    Ok(None)
}

async fn get_latency(shard_id: ShardId, shard_manager: Arc<ShardManager>) -> Option<Duration> {
    let runner = shard_manager.runners.lock().await;
    runner.get(&shard_id).and_then(|info| info.latency)
}
