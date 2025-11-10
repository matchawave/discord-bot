mod aliases;
mod bot;
mod channel;
mod command_options;
mod event;
mod http;
mod identifiers;
mod interaction;
mod parser;
mod prefix;
mod shard_manager;

pub use aliases::*;
pub use bot::*;
pub use channel::*;
pub use command_options::*;
pub use http::*;
pub use interaction::*;
pub use parser::*;
pub use prefix::*;
pub use shard_manager::*;

use serenity::{all::Context, async_trait};
use utils::{Parser, Pointer, debug};

use crate::HandlerFn;

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
        match U::extract(ctx, ev, p).await {
            Some(value) => Some(Some(value)),
            None => Some(None),
        }
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
        { async fn extract_tuple(ctx: &Context, action: &T, p: &Pointer<Parser>) -> Option<Self> { Some(($($ty::extract(ctx, action, p).await?,)*)) } }
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
