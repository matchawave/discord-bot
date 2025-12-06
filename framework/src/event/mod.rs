mod guild;
mod interactions;
mod message;
mod user;
mod voice_states;

use std::sync::Arc;

use colored::Colorize;
use dashmap::DashMap;
use serenity::{
    all::{Context, Event, RawEventHandler},
    async_trait,
};
use tokio::sync::mpsc::{self, Receiver, Sender};
use utils::{DiscordEvent, ElapsedTime, Parser, Pointer, error, info};

use crate::{
    HandlerFn,
    extractors::{DynHandler, ExtractorTuple, HandlerBuilder},
};

type EventCallback = Box<dyn DynHandler<Event, Output = ()> + Send + Sync>;
type EventCallbacks = Vec<EventCallback>;
type CallbackMap = DashMap<DiscordEvent, EventCallbacks>;

pub struct EventManager {
    senders: Vec<Sender<(Context, Event)>>,
    callbacks: Arc<CallbackMap>,
}

impl EventManager {
    pub fn new(shard_count: usize) -> Self {
        let mut senders = Vec::with_capacity(shard_count);
        let callbacks = Arc::new(DashMap::new());
        for _ in 0..shard_count {
            let (send, recv) = mpsc::channel(10_000);
            tokio::spawn(worker(recv, callbacks.clone()));
            senders.push(send);
        }
        Self { senders, callbacks }
    }

    #[warn(private_bounds)]
    pub fn add_handler<F, Args>(&self, event: DiscordEvent, callback: F)
    where
        F: HandlerFn<Args, ()> + Send + Sync + Copy + 'static,
        Args: ExtractorTuple<Event> + Send + Sync + 'static,
    {
        let handler = HandlerBuilder::<Event, ()>::build(callback);
        self.callbacks
            .entry(event)
            .or_default()
            .push(Box::new(handler));
    }
}

#[async_trait]
impl RawEventHandler for EventManager {
    async fn raw_event(&self, ctx: Context, event: Event) {
        let shard_id = ctx.shard_id.get() as usize;
        if let Some(sender) = self.senders.get(shard_id)
            && let Err(e) = sender.send((ctx, event)).await
        {
            error!("Error sending event to shard {}: {}", shard_id, e);
        }
    }
}

async fn worker(mut receiver: Receiver<(Context, Event)>, callbacks: Arc<CallbackMap>) {
    while let Some((ctx, event)) = receiver.recv().await {
        let shard_text = format!("(Shard {})", ctx.shard_id.get()).bold().purple();
        let seperator = "|".bold().white();
        let Some(event_name) = event.name() else {
            // info!("{} {} event received with no name", shard_text, seperator,);
            continue;
        };

        update_bot(&ctx, &event).await;
        cache_guild(&ctx, &event).await;

        if command_event(&ctx, &event).await {
            continue;
        }

        notify_startup(&ctx, &event).await;

        let name = DiscordEvent::from(&event);
        if let Some(funcs) = callbacks.get(&name)
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
                func.call(&ctx, &event, &parser).await;
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

async fn command_event(ctx: &Context, event: &Event) -> bool {
    let parser = Pointer::new(Parser::new(ctx.shard_id));
    let elapsed = ElapsedTime::new();
    if let Some(command_name) = match &event {
        Event::MessageCreate(e) => message::handle_command(ctx, &e.message, &parser).await,
        Event::MessageUpdate(e) => message::handle_edited_command(ctx, e, &parser).await,
        Event::InteractionCreate(e) => interactions::handle(ctx, &e.interaction, &parser).await,
        _ => None,
    } {
        info!(
            "command {} handled in {:?}ms",
            command_name,
            elapsed.elapsed_ms()
        );
        return true;
    }
    false
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
