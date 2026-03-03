mod guild;
mod interactions;
mod message;
mod user;
mod voice_states;

use std::{collections::HashMap, sync::Arc};

use colored::Colorize;
use serenity::{
    all::{Context, Event, RawEventHandler},
    async_trait,
};
use tokio::sync::mpsc::{self, Receiver, Sender};
use utils::{DiscordEvent, ElapsedTime, Parser, Pointer, ResponseError, error, info, warning};

use crate::{
    command::CommandEvent,
    extractors::ExtractorTuple,
    handler::{CallbackReturn, DynCallback, HandlerBuilder, HandlerFn},
};

pub type EventResult = Result<Option<()>, ResponseError>;

type EventCallbacks = Vec<Box<dyn DynCallback<Event, EventResult> + Send + Sync>>;
type CommandEventCallbacks = Vec<Box<dyn DynCallback<CommandEvent, EventResult> + Send + Sync>>;
type CallbackMap<T> = HashMap<DiscordEvent, T>;

#[derive(Default)]
pub struct EventManagerBuilder {
    event_callbacks: CallbackMap<EventCallbacks>,
    command_callbacks: CommandEventCallbacks,
}

impl EventManagerBuilder {
    #[warn(private_bounds)]
    pub fn add_event<F, U, Args>(mut self, event: DiscordEvent, callback: F) -> Self
    where
        F: HandlerFn<Args, U> + Send + Sync + Copy + 'static,
        Args: ExtractorTuple<Event> + Send + Sync + 'static,
        U: CallbackReturn<EventResult> + 'static,
    {
        let handler = HandlerBuilder::<Event, U>::build(callback);
        self.event_callbacks
            .entry(event)
            .or_default()
            .push(Box::new(handler));
        self
    }

    #[warn(private_bounds)]
    pub fn add_command<F, U, Args>(mut self, callback: F) -> Self
    where
        F: HandlerFn<Args, U> + Send + Sync + Copy + 'static,
        Args: ExtractorTuple<CommandEvent> + Send + Sync + 'static,
        U: CallbackReturn<EventResult> + 'static,
    {
        let handler = HandlerBuilder::<CommandEvent, U>::build(callback);
        self.command_callbacks.push(Box::new(handler));
        self
    }

    pub fn build(self, shard_count: usize) -> EventManager {
        let mut senders = Vec::with_capacity(shard_count);
        let events = Arc::new(self.event_callbacks);
        let commands = Arc::new(self.command_callbacks);
        for _ in 0..shard_count {
            let (send, recv) = mpsc::channel(10_000);
            tokio::spawn(worker(recv, events.clone(), commands.clone()));
            senders.push(send);
        }
        EventManager(senders)
    }
}

pub struct EventManager(Vec<Sender<(Context, Event)>>);

#[async_trait]
impl RawEventHandler for EventManager {
    async fn raw_event(&self, ctx: Context, event: Event) {
        let shard_id = ctx.shard_id.get() as usize;
        if let Some(sender) = self.0.get(shard_id)
            && let Err(e) = sender.send((ctx, event)).await
        {
            error!("Error sending event to shard {}: {}", shard_id, e);
        }
    }
}

async fn worker(
    mut receiver: Receiver<(Context, Event)>,
    events: Arc<CallbackMap<EventCallbacks>>,
    commands: Arc<CommandEventCallbacks>,
) {
    while let Some((ctx, event)) = receiver.recv().await {
        let shard_text = format!("(Shard {})", ctx.shard_id.get()).bold().purple();
        let seperator = "|".bold().white();
        let Some(event_name) = event.name() else {
            // info!("{} {} event received with no name", shard_text, seperator,);
            continue;
        };

        update_bot(&ctx, &event).await;
        cache_guild(&ctx, &event).await;

        match command_event(&ctx, &event).await {
            Ok(Some((command_event, parser))) => {
                for func in commands.iter() {
                    if let Some(Err(result)) = func.call(&ctx, &command_event, &parser).await {
                        match result {
                            ResponseError::Err(e, _) => error!("Command event: {e}"),
                            ResponseError::Warn(e, _) => warning!("Command event: {e}"),
                            ResponseError::Info(e) => info!("Command event: {e}"),
                        }
                    }
                }
                continue;
            }
            Ok(None) => {} // Not a command event, continue with normal processing
            Err(e) => match e {
                ResponseError::Err(e, _) => {
                    error!("{e}");
                    continue;
                }

                ResponseError::Warn(e, _) => warning!("Warning handling command event: {e}"),
                _ => {}
            },
        }

        notify_startup(&ctx, &event).await;
        let name = DiscordEvent::from(&event);
        if let Some(funcs) = events.get(&name)
            && !funcs.is_empty()
        {
            let elapsed = ElapsedTime::new();
            let parser = Pointer::new(Parser::new(ctx.shard_id));

            info!(
                "{} {} start {} event received",
                shard_text,
                seperator,
                event_name.bold().underline().green()
            );

            for func in funcs.iter() {
                if let Some(Err(result)) = func.call(&ctx, &event, &parser).await {
                    match result {
                        ResponseError::Err(e, _) => error!("{event_name} event: {e}"),
                        ResponseError::Warn(e, _) => warning!("{event_name} event: {e}"),
                        ResponseError::Info(e) => info!("{event_name} event: {e}"),
                    }
                }
            }

            info!(
                "{} {} end {} event handled in {:?}ms",
                shard_text.bold().purple(),
                seperator,
                event_name.bold().underline().green(),
                elapsed.elapsed_ms()
            );
        }

        handle_voice_state_update(&ctx, &event).await;
        delete_guilds(&ctx, &event).await;
    }
}

async fn command_event(
    ctx: &Context,
    event: &Event,
) -> Result<Option<(CommandEvent, Pointer<Parser>)>, ResponseError> {
    match event {
        Event::MessageCreate(e) => message::handle_command(ctx, e.message.clone()).await,
        Event::MessageUpdate(e) => message::handle_edited_command(ctx, e).await,
        Event::InteractionCreate(e) => interactions::handle(ctx, e.interaction.clone()).await,
        _ => Ok(None),
    }
}

async fn notify_startup(ctx: &Context, event: &Event) {
    if let Event::Ready(ready) = event {
        user::update_bot(ctx, &ready.ready.user).await;
    }
}

async fn update_bot(ctx: &Context, event: &Event) {
    if let Event::UserUpdate(e) = event {
        info!("Bot user updated");
        user::update_bot(ctx, &e.current_user).await;
    }
}

async fn cache_guild(ctx: &Context, event: &Event) {
    if let Event::GuildCreate(e) = event {
        guild::create(ctx, &e.guild).await;
    }
}

async fn delete_guilds(ctx: &Context, event: &Event) {
    if let Event::GuildDelete(e) = event {
        guild::delete(ctx, &e.guild).await;
    }
}

async fn handle_voice_state_update(ctx: &Context, event: &Event) {
    if let Event::VoiceStateUpdate(e) = event {
        voice_states::update(ctx, &e.voice_state).await;
    }
}

impl CallbackReturn<EventResult> for Result<(), ResponseError> {
    fn into_response(self: Box<Self>) -> Option<EventResult> {
        Some(self.map(|_| None))
    }
}

impl CallbackReturn<EventResult> for EventResult {
    fn into_response(self: Box<Self>) -> Option<EventResult> {
        Some(*self)
    }
}
