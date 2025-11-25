mod aliases;
mod bot;
mod command_options;
mod event;
mod http;
mod identifiers;
mod interaction;
mod parser;
mod shard_manager;

pub use aliases::*;
pub use bot::*;
pub use command_options::*;
pub use shard_manager::*;

use serenity::{all::Context, async_trait, prelude::TypeMapKey};
use utils::{Parser, Pointer};

use crate::{GlobalExtractable, HandlerFn};

#[async_trait]
pub trait Extractor<T>: Sized + Send + Sync + 'static {
    async fn extract(ctx: &Context, ev: &T, p: &Pointer<Parser>) -> Option<Self>;
}

#[async_trait]
impl<T, U> Extractor<T> for Option<U>
where
    T: Send + Sync + 'static,
    U: Extractor<T> + Send + Sync + 'static,
{
    async fn extract(ctx: &Context, ev: &T, p: &Pointer<Parser>) -> Option<Self> {
        Some(U::extract(ctx, ev, p).await)
    }
}

#[async_trait]
pub trait DynHandler<T>: Send + Sync {
    type Output: Send + Sync;
    async fn call(&self, ctx: &Context, ev: &T, p: &Pointer<Parser>) -> Option<Self::Output>;
}

#[async_trait]
pub trait ExtractorTuple<T>: Sized {
    async fn extract_tuple(ctx: &Context, action: &T, p: &Pointer<Parser>) -> Option<Self>;
}

pub(crate) struct Handler<T, U, F, Args>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
    F: HandlerFn<Args, U> + Send + Sync + Copy + 'static,
    Args: ExtractorTuple<T> + Send + Sync + 'static,
{
    callback: F,
    _type: std::marker::PhantomData<T>,
    _return: std::marker::PhantomData<U>,
    _args: std::marker::PhantomData<Args>,
}

pub(crate) struct HandlerBuilder<T, U>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
{
    _type: std::marker::PhantomData<T>,
    _return: std::marker::PhantomData<U>,
}

impl<T, U> HandlerBuilder<T, U>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
{
    pub fn build<F, Args>(callback: F) -> Handler<T, U, F, Args>
    where
        F: HandlerFn<Args, U> + Send + Sync + Copy + 'static,
        Args: ExtractorTuple<T> + Send + Sync + 'static,
    {
        Handler {
            callback,
            _type: std::marker::PhantomData,
            _return: std::marker::PhantomData,
            _args: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T, U, F, Args> DynHandler<T> for Handler<T, U, F, Args>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
    F: HandlerFn<Args, U> + Send + Sync + Copy + 'static,
    Args: ExtractorTuple<T> + Send + Sync + 'static,
{
    type Output = U;
    async fn call(&self, ctx: &Context, ev: &T, p: &Pointer<Parser>) -> Option<U> {
        if let Some(args) = Args::extract_tuple(ctx, ev, p).await {
            return Some(self.callback.call(args).await);
        }
        None
    }
}

macro_rules! impl_from_request_tuple {
    () => {
        #[serenity::async_trait]
        impl<T> ExtractorTuple<T> for ()
        where T: Send + Sync + 'static,
        { async fn extract_tuple(_ctx: &Context, _action: &T, _p: &Pointer<Parser>) -> Option<Self> { Some(()) } }
    };
    ($($ty:ident),*) => {
        #[serenity::async_trait]
        impl<T, $($ty,)*> ExtractorTuple<T> for ($($ty,)*)
        where T: Send + Sync + 'static, $($ty: Extractor<T> + Send + Sync + 'static,)*
        { async fn extract_tuple(ctx: &Context, action: &T, p: &Pointer<Parser>) -> Option<Self> {
            let result = (
                $(match $ty::extract(ctx, action, p).await {
                    Some(value) => value,
                    None => {
                        utils::debug!("[Extractor] : Failed to extract {}", std::any::type_name::<$ty>());
                        return None;
                    },
                },)*
            );
            Some(result)
        } }
    };
}
impl_from_request_tuple!();
impl_from_request_tuple!(A);
impl_from_request_tuple!(A, B);
impl_from_request_tuple!(A, B, C);
impl_from_request_tuple!(A, B, C, D);
impl_from_request_tuple!(A, B, C, D, E);
impl_from_request_tuple!(A, B, C, D, E, F);
impl_from_request_tuple!(A, B, C, D, E, F, G);
impl_from_request_tuple!(A, B, C, D, E, F, G, H);
impl_from_request_tuple!(A, B, C, D, E, F, G, H, I);
impl_from_request_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_from_request_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_from_request_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_from_request_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_from_request_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_from_request_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_from_request_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_from_request_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);

#[async_trait]
impl<T> GlobalExtractable for Pointer<T>
where
    Pointer<T>: TypeMapKey<Value = Pointer<T>>,
    T: Default + Send + Sync + 'static,
{
    fn init(map: &mut serenity::prelude::TypeMap) {
        map.insert::<Pointer<T>>(Pointer::new(T::default()));
    }

    async fn retrieve(map: &utils::DataType) -> Option<Self> {
        let data = map.read().await;
        data.get::<Pointer<T>>().cloned()
    }
}
