use serenity::{
    all::{Context, Event, Http, User},
    async_trait,
    prelude::TypeMapKey,
};

use utils::{DataType, Parser, Pointer, error};

use crate::{command::CommandAction, extractors::Extractor};

struct Bot;
impl TypeMapKey for Bot {
    type Value = User;
}

impl Bot {
    async fn http_get(http: impl AsRef<Http>) -> Option<User> {
        match http.as_ref().get_current_user().await {
            Ok(user) => Some(user.clone().into()),
            Err(e) => {
                error!("Failed to get current bot from discord api:\n{}", e);
                None
            }
        }
    }
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
impl Extractor<Event> for CurrentBot {
    async fn extract(ctx: &Context, _e: &Event, _p: &Pointer<Parser>) -> Option<Self> {
        CurrentBot::get(&ctx.data).await.map(CurrentBot)
    }
}

#[async_trait]
impl Extractor<CommandAction> for CurrentBot {
    async fn extract(ctx: &Context, _a: &CommandAction, _p: &Pointer<Parser>) -> Option<Self> {
        CurrentBot::get(&ctx.data).await.map(CurrentBot)
    }
}
