use serenity::all::{ClientBuilder, GatewayIntents, Http};

use crate::{
    command::{CommandManager, CommandManagerBuilder},
    event::{EventManager, EventManagerBuilder},
};

#[derive(Default)]
pub struct BotInstanceBuilder {
    event_manager: Option<EventManagerBuilder>,
    command_manager: Option<CommandManagerBuilder>,
    websocket_manager: Option<String>, // Placeholder for WebSocket manager builder
    permissions_manager: Option<String>, // Placeholder for permissions manager builder
}

impl BotInstanceBuilder {
    pub fn with_event_manager(mut self, event_manager: EventManagerBuilder) -> Self {
        self.event_manager = Some(event_manager);
        self
    }

    pub fn with_command_manager(mut self, command_manager: CommandManagerBuilder) -> Self {
        self.command_manager = Some(command_manager);
        self
    }

    pub fn build(
        self,
        api_url: String,
        bot_id: u64,
        bot_intents: GatewayIntents,
        shards: usize,
        token: String,
    ) -> ClientBuilder {
        let http = Http::new(&token);
        let event_manager = self
            .event_manager
            .unwrap_or_else(|| EventManagerBuilder::default())
            .build(shards);
        let command_manager = self
            .command_manager
            .unwrap_or_else(|| CommandManagerBuilder::default())
            .build();
        // Build the event manager and command manager here if needed, or pass them to the client
        ClientBuilder::new_with_http(http, bot_intents)
    }
}
