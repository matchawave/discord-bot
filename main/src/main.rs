use framework::extractors::ShardManagerContainer;
use serenity::{
    Client,
    all::{ApplicationId, ClientBuilder, GatewayIntents, Http},
    prelude::TypeMap,
};

use crate::{cache::set_sharded_cache, data::set_sharded_data, global::set_global};
mod cache;
mod commands;
mod data;
mod events;
mod extractors;
mod global;
mod processes;

#[tokio::main]
async fn main() {
    let shards = 1;

    let http = Http::new(env!("TOKEN"));
    let event_handler = events::create_event_handler(shards);
    let command_manager = commands::create_command_handler();

    let mut map = TypeMap::new();
    set_global(&mut map);
    set_sharded_data(shards, &mut map);
    set_sharded_cache(shards, &mut map);
    command_manager.set(&mut map);

    let client_builder = ClientBuilder::new_with_http(http, get_guild_intents())
        .application_id(ApplicationId::new(1340907937471660142))
        .type_map(map)
        .raw_event_handler(event_handler);

    match client_builder.await {
        Ok(mut client) => {
            command_manager.register(&client, false).await;
            set_shard_manager(&client).await;
            processes::start_background_processes(&client, shards).await;
            if let Err(e) = client.start_shards(shards as u32).await {
                println!("Error starting client: {:?}", e);
            }
        }
        Err(e) => println!("Error starting client: {:?}", e),
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
    let mut data = client.data.write().await;
    data.insert::<ShardManagerContainer>(client.shard_manager.clone());
}
