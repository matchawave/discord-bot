use serenity::{
    all::{ChannelId, Context, GuildChannel},
    async_trait,
    futures::channel,
};

use crate::{command::CommandAction, extractors::Extractor};

// #[async_trait]
// impl Extractor<CommandAction> for GuildChannel {
//     async fn extract(ctx: &Context, action: &CommandAction) -> Option<Self> {
//         let channel_id = ChannelId::extract(ctx, action).await?;
//     }
// }
