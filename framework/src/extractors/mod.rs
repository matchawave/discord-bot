mod bot;
mod command_options;
mod event;
mod http;
mod identifiers;
mod interaction;
mod parser;
mod shard_manager;

pub use bot::*;
pub use command_options::*;
pub use shard_manager::*;

use serenity::{all::Context, async_trait};
use utils::{Parser, Pointer};

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
pub trait EventExtractor<T>: Sized + Send + Sync + 'static {
    async fn extract_event(ev: &T) -> Option<Self>;
}

#[async_trait]
pub trait ContextExtractor: Sized + Send + Sync + 'static {
    async fn extract_context(ctx: &Context) -> Option<Self>;
}

#[async_trait]
pub trait ContextEventExtractor<T>: Sized + Send + Sync + 'static {
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self>;
}

#[async_trait]
pub trait ExtractorTuple<T>: Sized {
    async fn extract_tuple(ctx: &Context, action: &T, p: &Pointer<Parser>) -> Option<Self>;
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
