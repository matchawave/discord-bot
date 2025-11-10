use serenity::{
    all::{Context, Event},
    async_trait,
};
use utils::{Http, Parser, Pointer};

use crate::{command::CommandAction, extractors::Extractor};

#[async_trait]
impl Extractor<Event> for Http {
    async fn extract(ctx: &Context, _e: &Event, _p: &Pointer<Parser>) -> Option<Self> {
        Some(ctx.http.clone())
    }
}

#[async_trait]
impl Extractor<CommandAction> for Http {
    async fn extract(ctx: &Context, _a: &CommandAction, _p: &Pointer<Parser>) -> Option<Self> {
        Some(ctx.http.clone())
    }
}
