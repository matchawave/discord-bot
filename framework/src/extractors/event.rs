use serenity::{
    all::{Context, Event, Guild, UnavailableGuild},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::extractors::Extractor;

#[async_trait]
impl Extractor<Event> for Guild {
    async fn extract(_ctx: &Context, ev: &Event, _p: &Pointer<Parser>) -> Option<Self> {
        match ev {
            Event::GuildCreate(guild_create) => Some(guild_create.guild.clone()),
            _ => None,
        }
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
