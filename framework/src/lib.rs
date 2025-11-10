pub mod cache;
pub mod command;
pub mod data;
pub mod event;
pub mod extractors;
mod global;
pub mod processes;

use serenity::async_trait;

#[async_trait]
pub trait HandlerFn<T, U>: Send + Sync + 'static {
    async fn call(self, args: T) -> U;
}

macro_rules! impl_handler_fn {
    ($($ty:ident),+) => {
        #[serenity::async_trait]
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
