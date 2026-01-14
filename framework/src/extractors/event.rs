use serenity::{
    all::{Context, Event, Guild, Reaction, UnavailableGuild},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::extractors::Extractor;

#[async_trait]
impl Extractor<Event> for Guild {
    async fn extract(_ctx: &Context, ev: &Event, _p: &Pointer<Parser>) -> Option<Self> {
        let Event::GuildCreate(guild_create) = ev else {
            return None;
        };
        Some(guild_create.guild.clone())
    }
}

#[async_trait]
impl Extractor<Event> for UnavailableGuild {
    async fn extract(_ctx: &Context, ev: &Event, _p: &Pointer<Parser>) -> Option<Self> {
        match ev {
            Event::GuildDelete(guild_delete) => Some(guild_delete.guild),
            _ => None,
        }
    }
}

#[async_trait]
impl Extractor<Event> for Reaction {
    async fn extract(_ctx: &Context, ev: &Event, _p: &Pointer<Parser>) -> Option<Self> {
        match ev {
            Event::ReactionAdd(reaction_add) => Some(reaction_add.reaction.clone()),
            Event::ReactionRemove(reaction_remove) => Some(reaction_remove.reaction.clone()),
            Event::ReactionRemoveAll(_reaction_remove_all) => {
                // Not enough info to create a Reaction
                None
            }
            Event::ReactionRemoveEmoji(reaction_remove_emoji) => {
                Some(reaction_remove_emoji.reaction.clone())
            }
            _ => None,
        }
    }
}
