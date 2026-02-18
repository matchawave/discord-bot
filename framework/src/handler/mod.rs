use serenity::{all::Context, async_trait};
use utils::{Parser, Pointer};

use crate::extractors::ExtractorTuple;

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
pub trait DynHandler<T>: Send + Sync {
    type Output: Send + Sync;
    async fn call(&self, ctx: &Context, ev: &T, p: &Pointer<Parser>) -> Option<Self::Output>;
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

#[async_trait]
/// A trait for function handlers that can be called asynchronously
/// with specific argument and return types.
/// T: The type of the arguments the function takes.
/// U: The return type of the function.
pub trait HandlerFn<T, U>: Send + Sync + Copy + 'static {
    async fn call(self, args: T) -> U;
}

pub trait CallbackReturn<T>: Send + Sync {
    fn into_response(self: Box<Self>) -> Option<T>;
}

impl<T> CallbackReturn<T> for () {
    fn into_response(self: Box<Self>) -> Option<T> {
        None
    }
}

#[async_trait]
pub trait DynCallback<E, U>: Send + Sync
where
    E: Send + Sync,
    U: Send + Sync,
{
    async fn call(&self, ctx: &Context, event: &E, p: &Pointer<utils::Parser>) -> Option<U>;
}

#[async_trait]
impl<E, U, D> DynCallback<E, U> for D
where
    E: Send + Sync,
    U: Send + Sync,
    D: DynHandler<E> + 'static,
    D::Output: CallbackReturn<U>,
{
    async fn call(&self, ctx: &Context, event: &E, p: &Pointer<utils::Parser>) -> Option<U> {
        if let Some(result) = DynHandler::call(self, ctx, event, p).await {
            return Box::new(result).into_response();
        }
        None
    }
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
