use std::sync::Arc;

use framework::{ShardData, extractors::ShardManagerContainer};
use serenity::{
    Client,
    all::{ApplicationId, ClientBuilder, GatewayIntents, Http, ShardManager},
    prelude::TypeMap,
};
use utils::{DataType, error};

use crate::global::{backend_http::BackendHttp, set_global};
mod cache;
mod commands;
mod configs;
mod data;
mod events;
mod global;
mod processes;
mod websocket;

#[tokio::main]
async fn main() {
    let shards = 1;
    let token = env!("TOKEN");
    let api_url = env!("BACKEND_URL");
    let bot_id = get_bot_id();
    let bot_intent = get_guild_intents();

    let http = Http::new(token);
    let mut data = TypeMap::new();

    let event_handler = events::create_event_handler(shards); // Create the event handler for the bot
    let command_manager = commands::create_command_handler(); // Command manager for registering and handling commands
    let backend_http = BackendHttp::new(token, api_url); // This is for communicating with the backend serverlet backend_http = BackendHttp::new(token, api_url); // This is for communicating with the backend server
    backend_http.set_shards(shards as u32).await; // Set the number of shards in the backend
    let websocket = websocket::get_websocket_connection().build(api_url, bot_id, token); // WebSocket connection to the backend
    let process_manager = processes::get_bg_process_manager(&mut data, websocket).await; // Background process manager

    set_global(backend_http, &mut data);
    ShardData::init(shards, &mut data);
    command_manager.set(&mut data);

    let client_builder = ClientBuilder::new_with_http(http, bot_intent)
        .application_id(bot_id)
        .type_map(data)
        .raw_event_handler(event_handler);

    let mut client = match client_builder.await {
        Ok(c) => {
            set_shard_manager(&c).await;
            c
        }
        Err(e) => {
            error!("Error creating client: {:?}", e);
            return;
        }
    };

    process_manager.init_loop(&client.http, &client.data);
    command_manager.register(&client.http, true).await;

    if let Err(e) = client.start_shards(shards as u32).await {
        error!("Error starting client: {:?}", e);
    }
}

fn get_guild_intents() -> GatewayIntents {
    GatewayIntents::GUILDS
        .union(GatewayIntents::GUILD_MESSAGES)
        .union(GatewayIntents::GUILD_MESSAGE_REACTIONS)
        .union(GatewayIntents::GUILD_VOICE_STATES)
        .union(GatewayIntents::GUILD_PRESENCES)
        .union(GatewayIntents::GUILD_MEMBERS)
        .union(GatewayIntents::GUILD_MODERATION)
        .union(GatewayIntents::GUILD_EMOJIS_AND_STICKERS)
        .union(GatewayIntents::GUILD_INTEGRATIONS)
        .union(GatewayIntents::GUILD_WEBHOOKS)
        .union(GatewayIntents::GUILD_INVITES)
        .union(GatewayIntents::GUILD_SCHEDULED_EVENTS)
        .union(GatewayIntents::MESSAGE_CONTENT)
}

async fn set_shard_manager(client: &Client) {
    let data = client.data.clone();
    (data.write().await).insert::<ShardManagerContainer>(client.shard_manager.clone());
}

fn get_bot_id() -> ApplicationId {
    env!("APPLICATION_ID")
        .parse::<u64>()
        .map(ApplicationId::new)
        .unwrap()
}
