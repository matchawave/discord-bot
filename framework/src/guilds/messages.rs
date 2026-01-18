use std::time::Duration;

use moka::future::Cache;
use serenity::{
    all::{ChannelId, Context, Event, GuildId, Message, MessageId},
    async_trait,
    prelude::TypeMapKey,
};
use utils::{HttpType, Parser, Pointer, debug, error};

use crate::{
    ShardData,
    command::CommandAction,
    extractors::{ContextEventExtractor, EventExtractor, Extractor},
    guilds::HTTPGetter,
};

pub struct Messages(pub Cache<(ChannelId, MessageId), Message>);

impl TypeMapKey for Messages {
    type Value = Cache<(ChannelId, MessageId), Message>;
}

impl Default for Messages {
    fn default() -> Self {
        Messages(
            Cache::builder()
                .max_capacity(100_000)
                .time_to_live(Duration::from_hours(12))
                .build(),
        )
    }
}

#[async_trait]
impl<T> ContextEventExtractor<T> for Messages
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self> {
        let data = ShardData::get(ctx.shard_id, &ctx.data).await?;
        let guild_id = GuildId::extract_event(ev).await?;
        let map = data.guilds.map(guild_id).await?;
        map.read().await.get::<Messages>().cloned().map(Messages)
    }
}

#[async_trait]
impl<T> Extractor<T> for Messages
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract(ctx: &Context, ev: &T, _: &Pointer<utils::Parser>) -> Option<Self> {
        Messages::extract_context_event(ctx, ev).await
    }
}

#[async_trait]
impl super::HTTPGetter<(ChannelId, MessageId), Message> for Messages {
    async fn fetch(&self, http: &HttpType, key: (ChannelId, MessageId)) -> Option<Message> {
        let (channel_id, message_id) = key;
        match self.0.get(&key).await {
            Some(message) => Some(message),
            None => match channel_id.message(http, message_id).await {
                Ok(message) => {
                    self.0.insert(key, message.clone()).await;
                    Some(message)
                }
                Err(err) => {
                    error!(
                        "Failed to fetch message {} in channel {}: {}",
                        message_id, channel_id, err
                    );
                    None
                }
            },
        }
    }
}

#[async_trait]
impl Extractor<Event> for Message
where
    ChannelId: EventExtractor<Event>,
    MessageId: EventExtractor<Event>,
{
    async fn extract(ctx: &Context, ev: &Event, _: &Pointer<Parser>) -> Option<Self> {
        let messages = Messages::extract_context_event(ctx, ev).await?;
        match ev {
            Event::MessageCreate(event) => {
                let Messages(messages) = messages;
                if event.message.author.bot {
                    return None;
                }
                if messages
                    .get(&(event.message.channel_id, event.message.id))
                    .await
                    .is_some()
                {
                    debug!(
                        "Message id {} already in cache, skipping insert",
                        event.message.id
                    );
                    return Some(event.message.clone());
                }
                debug!("Inserting message id {} into cache", event.message.id);
                messages
                    .insert(
                        (event.message.channel_id, event.message.id),
                        event.message.clone(),
                    )
                    .await;
                return Some(event.message.clone());
            }
            Event::MessageUpdate(event) => {
                let mut msg: Message = Message::default();
                event.apply_to_message(&mut msg);
                return Some(msg);
            }
            _ => {}
        }

        let channel_id = ChannelId::extract_event(ev).await?;
        let message_id = MessageId::extract_event(ev).await?;
        messages.fetch(&ctx.http, (channel_id, message_id)).await
    }
}

#[async_trait]
impl Extractor<CommandAction> for Message {
    async fn extract(_ctx: &Context, action: &CommandAction, _p: &Pointer<Parser>) -> Option<Self> {
        if let CommandAction::Message(msg) = action {
            return Some(*msg.clone());
        }
        None
    }
}
