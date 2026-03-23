use framework::{
    CacheExtractable,
    extractors::{ContextEventExtractor, ContextExtractor, EventExtractor, Extractor},
};
use moka::future::Cache;
use serenity::{
    all::{ChannelId, GuildId, Message, Reaction},
    prelude::TypeMapKey,
};

macro_rules! snipe_builder {
    ($($name:ident, $base:ty;)*) => {
        $(
            #[derive(Clone,  CacheExtractable)]
            #[cache(live="2h", capacity=100000)]
            pub struct $name(pub Cache<ChannelId, utils::Pointer<Vec<$base>>>);

            impl TypeMapKey for $name {
                type Value = Cache<ChannelId, utils::Pointer<Vec<$base>>>;
            }
            #[serenity::async_trait]
            impl<T> ContextEventExtractor<T> for $name
            where
                T: Send + Sync + 'static,
                GuildId: EventExtractor<T>,
            {
                async fn extract_context_event(
                    ctx: &serenity::all::Context,
                    ev: &T,
                ) -> Option<Self> {
                    let data = framework::ShardData::extract_context(&ctx).await?;
                    let guild_id = GuildId::extract_event(ev).await?;
                    let guild_data = data.guilds.map(guild_id).await?;
                    (guild_data.read().await.get::<$name>())
                        .cloned()
                        .map($name)
                }
            }

            #[serenity::async_trait]
            impl<T> Extractor<T> for $name
            where
                T: Send + Sync + 'static,
                GuildId: EventExtractor<T>,
            {
                async fn extract(
                    ctx: &serenity::all::Context,
                    ev: &T,
                    _: &utils::Pointer<utils::Parser>,
                ) -> Option<Self> {
                    Self::extract_context_event(ctx, ev).await
                }
            }
        )*
    };
}

snipe_builder!(
    Snipes, Message;
    EditSnipes, Message;
    ReactionSnipes, Reaction;
);
