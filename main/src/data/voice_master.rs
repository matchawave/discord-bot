use std::collections::HashMap;

use framework::{
    extractors::{ContextExtractor, EventExtractor, Extractor},
    guilds::Guilds,
};

use serenity::{
    all::{ChannelId, Context, GuildId, UserId},
    async_trait,
};
use utils::{Parser, Pointer, error};

use crate::configs::voice::VoiceMasterConfig;
type ActiveMap = HashMap<ChannelId, UserId>; // Maps active voice channels to their owners

pub struct VoiceMaster {
    pub config: Pointer<VoiceMasterConfig>,
    pub actives: Pointer<ActiveMap>,
}

#[async_trait]
impl<T> Extractor<T> for VoiceMaster
where
    GuildId: EventExtractor<T>,
    T: Send + Sync + 'static,
{
    async fn extract(ctx: &Context, ev: &T, _: &Pointer<Parser>) -> Option<VoiceMaster> {
        let guild_id = GuildId::extract_event(ev).await?;
        let guilds = Guilds::extract_context(ctx).await?;
        let config = guilds.get_ptr::<VoiceMasterConfig>(guild_id).await?;
        let actives = match guilds.get_ptr::<ActiveMap>(guild_id).await {
            Some(ptr) => ptr,
            None => match guilds
                .insert_ptr::<ActiveMap>(guild_id, ActiveMap::new())
                .await
            {
                Ok(ptr) => ptr,
                Err(e) => {
                    error!(
                        "(voice master) Failed to insert active map for guild {}: {}",
                        guild_id, e
                    );
                    return None;
                }
            },
        };
        Some(VoiceMaster { config, actives })
    }
}
