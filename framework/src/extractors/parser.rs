use serenity::{all::Event, async_trait};
use utils::{Parser, Pointer};

use crate::extractors::Extractor;

#[async_trait]
impl Extractor<Event> for Pointer<Parser> {
    async fn extract(
        _ctx: &serenity::all::Context,
        _ev: &Event,
        p: &Pointer<Parser>,
    ) -> Option<Self> {
        Some(p.clone())
    }
}
