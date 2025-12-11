use framework::{
    extractors::{ContextExtractor, EventExtractor, Extractor},
    guilds::Guilds,
};

use serenity::{
    all::{Context, GuildId},
    async_trait,
};
use utils::{Parser, Pointer};

use crate::configs::VoiceMasterConfig;

pub struct VoiceMaster(pub Pointer<VoiceMasterConfig>);

#[async_trait]
impl<T> Extractor<T> for VoiceMaster
where
    GuildId: EventExtractor<T>,
    T: Send + Sync + 'static,
{
    async fn extract(ctx: &Context, ev: &T, _: &Pointer<Parser>) -> Option<VoiceMaster> {
        let guild_id = GuildId::extract_event(ev).await?;
        let guilds = Guilds::extract_context(ctx).await?;
        (guilds.get_ptr::<VoiceMasterConfig>(guild_id).await).map(VoiceMaster)
    }
}
