use serenity::{
    all::{ChannelId, Context, Event, GuildId, Interaction, MessageId, ShardId, UserId},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::{
    command::{CommandAction, CommandEvent},
    extractors::{ContextExtractor, EventExtractor, Extractor},
};

#[async_trait]
impl ContextExtractor for ShardId {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        Some(ctx.shard_id)
    }
}

#[async_trait]
impl EventExtractor<Event> for GuildId {
    async fn extract_event(ev: &Event) -> Option<Self> {
        match ev {
            Event::GuildCreate(env) => Some(env.guild.id),
            Event::GuildUpdate(env) => Some(env.guild.id),
            Event::GuildDelete(env) => Some(env.guild.id),
            Event::GuildBanAdd(env) => Some(env.guild_id),
            Event::GuildBanRemove(env) => Some(env.guild_id),
            Event::GuildEmojisUpdate(env) => Some(env.guild_id),
            Event::GuildIntegrationsUpdate(env) => Some(env.guild_id),
            Event::GuildMemberAdd(env) => Some(env.member.guild_id),
            Event::GuildMemberRemove(env) => Some(env.guild_id),
            Event::GuildMemberUpdate(env) => Some(env.guild_id),
            Event::GuildMembersChunk(env) => Some(env.guild_id),
            Event::GuildRoleCreate(env) => Some(env.role.guild_id),
            Event::GuildRoleUpdate(env) => Some(env.role.guild_id),
            Event::GuildRoleDelete(env) => Some(env.guild_id),
            Event::GuildAuditLogEntryCreate(env) => Some(env.guild_id),
            Event::GuildScheduledEventCreate(env) => Some(env.event.guild_id),
            Event::GuildScheduledEventUpdate(env) => Some(env.event.guild_id),
            Event::GuildScheduledEventDelete(env) => Some(env.event.guild_id),
            Event::GuildScheduledEventUserAdd(env) => Some(env.guild_id),
            Event::GuildScheduledEventUserRemove(env) => Some(env.guild_id),
            Event::ChannelCreate(env) => Some(env.channel.guild_id),
            Event::ChannelUpdate(env) => Some(env.channel.guild_id),
            Event::ChannelDelete(env) => Some(env.channel.guild_id),
            Event::ChannelPinsUpdate(env) => env.guild_id,
            Event::ThreadCreate(env) => Some(env.thread.guild_id),
            Event::ThreadUpdate(env) => Some(env.thread.guild_id),
            Event::ThreadDelete(env) => Some(env.thread.guild_id),
            Event::ThreadListSync(env) => Some(env.guild_id),
            Event::ThreadMemberUpdate(env) => env.member.guild_id,
            Event::ThreadMembersUpdate(env) => Some(env.guild_id),
            Event::AutoModActionExecution(env) => Some(env.execution.guild_id),
            Event::EntitlementCreate(env) => env.entitlement.guild_id,
            Event::EntitlementDelete(env) => env.entitlement.guild_id,
            Event::EntitlementUpdate(env) => env.entitlement.guild_id,
            Event::MessageCreate(env) => env.message.guild_id,
            Event::MessageUpdate(env) => env.guild_id,
            Event::MessageDelete(env) => env.guild_id,
            Event::MessageDeleteBulk(env) => env.guild_id,
            Event::VoiceChannelStatusUpdate(env) => Some(env.guild_id),
            Event::VoiceStateUpdate(env) => env.voice_state.guild_id,
            Event::VoiceServerUpdate(env) => env.guild_id,
            Event::StageInstanceCreate(env) => Some(env.stage_instance.guild_id),
            Event::StageInstanceUpdate(env) => Some(env.stage_instance.guild_id),
            Event::StageInstanceDelete(env) => Some(env.stage_instance.guild_id),
            Event::InteractionCreate(env) => match &env.interaction {
                Interaction::Command(cmd) => cmd.guild_id,
                Interaction::Component(comp) => comp.guild_id,
                Interaction::Autocomplete(auto) => auto.guild_id,
                Interaction::Modal(modal) => modal.guild_id,
                _ => None,
            },
            Event::ReactionAdd(react) => react.reaction.guild_id,
            Event::ReactionRemove(react) => react.reaction.guild_id,
            Event::ReactionRemoveAll(env) => env.guild_id,
            Event::ReactionRemoveEmoji(env) => env.reaction.guild_id,

            Event::Unknown(_env) => None,
            _ => None,
        }
    }
}

#[async_trait]
impl EventExtractor<CommandEvent> for GuildId {
    async fn extract_event(action: &CommandEvent) -> Option<Self> {
        match &action.action {
            CommandAction::Interaction(i) => i.guild_id,
            CommandAction::Message(m) => m.guild_id,
        }
    }
}

#[async_trait]
impl EventExtractor<Event> for ChannelId {
    async fn extract_event(ev: &Event) -> Option<Self> {
        match ev {
            Event::MessageCreate(env) => Some(env.message.channel_id),
            Event::MessageUpdate(env) => Some(env.channel_id),
            Event::MessageDelete(env) => Some(env.channel_id),
            Event::MessageDeleteBulk(env) => Some(env.channel_id),
            Event::ChannelCreate(env) => Some(env.channel.id),
            Event::ChannelUpdate(env) => Some(env.channel.id),
            Event::ChannelDelete(env) => Some(env.channel.id),
            Event::ChannelPinsUpdate(env) => Some(env.channel_id),
            Event::ThreadCreate(env) => Some(env.thread.id),
            Event::ThreadUpdate(env) => Some(env.thread.id),
            Event::ThreadDelete(env) => Some(env.thread.id),
            Event::ThreadListSync(_env) => None, // * VEC<ChannelId>
            Event::ThreadMembersUpdate(env) => Some(env.id),
            Event::InteractionCreate(env) => match &env.interaction {
                Interaction::Command(cmd) => Some(cmd.channel_id),
                Interaction::Component(comp) => Some(comp.channel_id),
                Interaction::Autocomplete(auto) => Some(auto.channel_id),
                Interaction::Modal(modal) => Some(modal.channel_id),
                _ => None,
            },
            _ => None,
        }
    }
}

#[async_trait]
impl EventExtractor<CommandEvent> for ChannelId {
    async fn extract_event(ev: &CommandEvent) -> Option<Self> {
        match &ev.action {
            CommandAction::Message(m) => Some(m.channel_id),
            CommandAction::Interaction(i) => Some(i.channel_id),
        }
    }
}

#[async_trait]
impl EventExtractor<Event> for Vec<ChannelId> {
    async fn extract_event(ev: &Event) -> Option<Self> {
        match ev {
            Event::ThreadListSync(env) => env.channel_ids.clone(),
            _ => None,
        }
    }
}

#[async_trait]
impl EventExtractor<Event> for MessageId {
    async fn extract_event(ev: &Event) -> Option<Self> {
        match ev {
            Event::MessageCreate(env) => Some(env.message.id),
            Event::MessageUpdate(env) => Some(env.id),
            Event::MessageDelete(env) => Some(env.message_id),
            Event::MessageDeleteBulk(_env) => None, // *VEC<MessageId>
            Event::ReactionAdd(env) => Some(env.reaction.message_id),
            Event::ReactionRemove(env) => Some(env.reaction.message_id),
            Event::ReactionRemoveAll(env) => Some(env.message_id),
            Event::ReactionRemoveEmoji(env) => Some(env.reaction.message_id),
            _ => None,
        }
    }
}

#[async_trait]
impl EventExtractor<Event> for Vec<MessageId> {
    async fn extract_event(ev: &Event) -> Option<Self> {
        match ev {
            Event::MessageDeleteBulk(env) => Some(env.ids.clone()),
            _ => None,
        }
    }
}

#[async_trait]
impl EventExtractor<CommandEvent> for MessageId {
    async fn extract_event(action: &CommandEvent) -> Option<Self> {
        match &action.action {
            CommandAction::Message(m) => Some(m.id),
            _ => None,
        }
    }
}

#[async_trait]
impl EventExtractor<Event> for UserId {
    async fn extract_event(ev: &Event) -> Option<Self> {
        match ev {
            Event::GuildMemberAdd(env) => Some(env.member.user.id),
            Event::GuildMemberRemove(env) => Some(env.user.id),
            Event::GuildMemberUpdate(env) => Some(env.user.id),
            // Event::GuildMembersChunk(env) => None, // * VEC<UserId>
            Event::MessageCreate(env) => Some(env.message.author.id),
            Event::MessageUpdate(env) => env.author.as_ref().map(|u| u.id),
            // Event::MessageDelete(env) => None,
            // Event::MessageDeleteBulk(env) => None,
            Event::InteractionCreate(env) => match &env.interaction {
                Interaction::Command(cmd) => Some(cmd.user.id),
                Interaction::Component(comp) => Some(comp.user.id),
                Interaction::Autocomplete(auto) => Some(auto.user.id),
                Interaction::Modal(modal) => Some(modal.user.id),
                _ => None,
            },
            Event::IntegrationCreate(env) => env.integration.user.as_ref().map(|u| u.id),
            Event::IntegrationUpdate(env) => env.integration.user.as_ref().map(|u| u.id),
            Event::ReactionAdd(env) => env.reaction.user_id,
            Event::ReactionRemove(env) => env.reaction.user_id,
            Event::VoiceStateUpdate(env) => Some(env.voice_state.user_id),
            _ => None,
        }
    }
}

#[async_trait]
impl EventExtractor<CommandEvent> for UserId {
    async fn extract_event(action: &CommandEvent) -> Option<Self> {
        match &action.action {
            CommandAction::Message(m) => Some(m.author.id),
            CommandAction::Interaction(i) => Some(i.user.id),
        }
    }
}

#[async_trait]
impl<T> Extractor<T> for ShardId {
    async fn extract(ctx: &Context, _ev: &T, _p: &Pointer<Parser>) -> Option<Self> {
        ShardId::extract_context(ctx).await
    }
}

#[async_trait]
impl<T> Extractor<T> for GuildId
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract(_: &Context, ev: &T, _: &Pointer<Parser>) -> Option<Self> {
        GuildId::extract_event(ev).await
    }
}

#[async_trait]
impl<T> Extractor<T> for ChannelId
where
    T: Send + Sync + 'static,
    ChannelId: EventExtractor<T>,
{
    async fn extract(_: &Context, ev: &T, _: &Pointer<Parser>) -> Option<Self> {
        ChannelId::extract_event(ev).await
    }
}

#[async_trait]
impl<T> Extractor<T> for Vec<ChannelId>
where
    T: Send + Sync + 'static,
    Vec<ChannelId>: EventExtractor<T>,
{
    async fn extract(_: &Context, ev: &T, _: &Pointer<Parser>) -> Option<Self> {
        Vec::<ChannelId>::extract_event(ev).await
    }
}

#[async_trait]
impl<T> Extractor<T> for MessageId
where
    T: Send + Sync + 'static,
    MessageId: EventExtractor<T>,
{
    async fn extract(_: &Context, ev: &T, _: &Pointer<Parser>) -> Option<Self> {
        MessageId::extract_event(ev).await
    }
}

#[async_trait]
impl<T> Extractor<T> for Vec<MessageId>
where
    T: Send + Sync + 'static,
    Vec<MessageId>: EventExtractor<T>,
{
    async fn extract(_: &Context, ev: &T, _: &Pointer<Parser>) -> Option<Self> {
        Vec::<MessageId>::extract_event(ev).await
    }
}

#[async_trait]
impl<T> Extractor<T> for UserId
where
    T: Send + Sync + 'static,
    UserId: EventExtractor<T>,
{
    async fn extract(_: &Context, ev: &T, _: &Pointer<Parser>) -> Option<Self> {
        UserId::extract_event(ev).await
    }
}
