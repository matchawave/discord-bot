pub mod command;
pub mod data;
pub mod event;
pub mod extractors;
pub mod global;
pub mod guilds;
pub mod processes;
pub mod websocket;

pub use macros::*;
use utils::{DataType, Pointer};

use std::sync::Arc;

use serenity::{
    all::{Context, ShardId, User},
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};

use crate::{
    extractors::{ContextEventExtractor, ContextExtractor, Extractor},
    guilds::Guilds,
};

pub trait Extractable {
    fn init(map: &mut TypeMap);
    fn retrieve(map: &Arc<TypeMap>) -> Option<Self>
    where
        Self: Sized;
}

#[async_trait]
/// A trait for function handlers that can be called asynchronously
/// with specific argument and return types.
/// T: The type of the arguments the function takes.
/// U: The return type of the function.
pub trait HandlerFn<T, U>: Send + Sync + Copy + 'static {
    async fn call(self, args: T) -> U;
}

macro_rules! impl_handler_fn {
    ($($ty:ident),+) => {
        #[serenity::async_trait]
        #[allow(non_snake_case)]
        impl<Func, Fut, U, $($ty,)+> HandlerFn<($($ty,)+), U> for Func
        where
            Func: FnOnce($($ty,)+) -> Fut + Send + Sync + Copy + 'static,
            Fut: std::future::Future<Output = U> + Send,
            U: Send + 'static,
            $($ty: Send + 'static,)+
        {
            async fn call(self, ($($ty,)+): ($($ty,)+)) -> U {
                // Call the function with the extracted arguments
                (self)($($ty,)+).await
            }
        }
    };
}

#[async_trait]

impl<Func, Fut, U> HandlerFn<(), U> for Func
where
    Func: FnOnce() -> Fut + Send + Sync + Copy + 'static,
    Fut: std::future::Future<Output = U> + Send,
    U: Send + 'static,
{
    async fn call(self, _args: ()) -> U {
        (self)().await
    }
}

impl_handler_fn!(A);
impl_handler_fn!(A, B);
impl_handler_fn!(A, B, C);
impl_handler_fn!(A, B, C, D);
impl_handler_fn!(A, B, C, D, E);
impl_handler_fn!(A, B, C, D, E, F);
impl_handler_fn!(A, B, C, D, E, F, G);
impl_handler_fn!(A, B, C, D, E, F, G, H);
impl_handler_fn!(A, B, C, D, E, F, G, H, I);
impl_handler_fn!(A, B, C, D, E, F, G, H, I, J);
impl_handler_fn!(A, B, C, D, E, F, G, H, I, J, K);
impl_handler_fn!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_handler_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_handler_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_handler_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_handler_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_handler_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);

#[derive(Clone)]
pub struct ShardData {
    pub guilds: Guilds,
    default_prefix: String,
    bot: Pointer<Option<User>>,
}

impl Default for ShardData {
    fn default() -> Self {
        Self {
            guilds: Guilds::default(),
            default_prefix: "!".to_string(),
            bot: Pointer::default(),
        }
    }
}

impl TypeMapKey for ShardData {
    type Value = Pointer<Vec<ShardData>>;
}

impl ShardData {
    pub fn init(shards: usize, data: &mut TypeMap) {
        let mut data_vec = Vec::with_capacity(shards);
        for _ in 0..shards {
            data_vec.push(ShardData::default());
        }
        data.insert::<ShardData>(Pointer::new(data_vec));
    }

    pub async fn get(shard_id: ShardId, data: &DataType) -> Option<ShardData> {
        let data = data.read().await;
        let data = data.get::<ShardData>()?;
        let shard_id = shard_id.get() as usize;

        data.read().await.get(shard_id).cloned()
    }
}

#[async_trait]
impl ContextExtractor for ShardData {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        ShardData::get(ctx.shard_id, &ctx.data).await
    }
}

#[async_trait]
impl<T> Extractor<T> for ShardData
where
    T: Send + Sync + 'static,
{
    async fn extract(ctx: &Context, _: &T, _: &Pointer<utils::Parser>) -> Option<Self> {
        ShardData::extract_context(ctx).await
    }
}
