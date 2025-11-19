use std::sync::Arc;

use serenity::{
    all::{Context, Event},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::{command::CommandAction, extractors::Extractor};

#[async_trait]
impl Extractor<Event> for Arc<serenity::http::Http> {
    async fn extract(ctx: &Context, _e: &Event, _p: &Pointer<Parser>) -> Option<Self> {
        Some(ctx.http.clone())
    }
}

#[async_trait]
impl Extractor<CommandAction> for Arc<serenity::http::Http> {
    async fn extract(ctx: &Context, _a: &CommandAction, _p: &Pointer<Parser>) -> Option<Self> {
        Some(ctx.http.clone())
    }
}
