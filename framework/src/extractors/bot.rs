use serenity::{
    all::{Context, User},
    async_trait,
    prelude::TypeMapKey,
};

use utils::{DataType, Parser, Pointer};

use crate::extractors::{ContextExtractor, Extractor};

struct Bot;
impl TypeMapKey for Bot {
    type Value = User;
}

pub struct CurrentBot(pub User);

impl CurrentBot {
    pub async fn set(data: &DataType, bot: User) {
        let mut data = data.write().await;
        data.insert::<Bot>(bot);
    }
    pub async fn get(data: &DataType) -> Option<User> {
        let data = data.read().await;
        data.get::<Bot>().cloned()
    }
    pub async fn is_set(data: &DataType) -> bool {
        let data = data.read().await;
        data.contains_key::<Bot>()
    }
}

#[async_trait]
impl ContextExtractor for CurrentBot {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        CurrentBot::get(&ctx.data).await.map(CurrentBot)
    }
}

#[async_trait]
impl<T> Extractor<T> for CurrentBot
where
    T: Send + Sync,
{
    async fn extract(ctx: &Context, _e: &T, _p: &Pointer<Parser>) -> Option<Self> {
        Self::extract_context(ctx).await
    }
}
