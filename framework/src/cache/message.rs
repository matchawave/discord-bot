use std::sync::Arc;

use serenity::{
    all::{ChannelId, Context, Event, Message, MessageId},
    async_trait,
};
use utils::{Parser, Pointer, debug, error};

use crate::{cache::HTTPGetter, cached, command::CommandAction, extractors::Extractor};

cached!(Messages, Message, (ChannelId, MessageId));

#[async_trait]
impl HTTPGetter<(ChannelId, MessageId), Message> for Messages {
    async fn fetch(
        &self,
        http: &Arc<serenity::http::Http>,
        key: (ChannelId, MessageId),
    ) -> Option<Message> {
        let (channel_id, message_id) = key;
        match self.0.read().await.get(&key).await {
            Some(message) => Some(message.clone()),
            None => match channel_id.message(http, message_id).await {
                Ok(message) => {
                    self.insert(key, message.clone()).await;
                    Some(message)
                }
                Err(err) => {
                    error!(
                        "Failed to fetch message {} from channel {}: {}",
                        message_id, channel_id, err
                    );
                    None
                }
            },
        }
    }
}

#[async_trait]
impl Extractor<Event> for Message {
    async fn extract(ctx: &Context, ev: &Event, p: &Pointer<Parser>) -> Option<Self> {
        let messages = Messages::extract(ctx, ev, p).await?;
        match ev {
            Event::MessageCreate(event) => {
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

        let (channel_id, message_id) = get_ids(ctx, ev, p).await?;
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

async fn get_ids<T>(ctx: &Context, ev: &T, p: &Pointer<Parser>) -> Option<(ChannelId, MessageId)>
where
    ChannelId: Extractor<T>,
    MessageId: Extractor<T>,
{
    let channel_id = ChannelId::extract(ctx, ev, p).await?;
    let message_id = MessageId::extract(ctx, ev, p).await?;
    Some((channel_id, message_id))
}
