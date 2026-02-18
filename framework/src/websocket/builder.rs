use serde_json::Value;
use serenity::async_trait;
use utils::{DataType, HttpType, error};

use crate::handler::HandlerFn;

pub(super) struct WSHandler<F, T>
where
    F: HandlerFn<(T, DataType, HttpType), ()> + Send + Sync + Copy + 'static,
    T: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    callback: F,
    _type: std::marker::PhantomData<T>,
}

/// WebSocket-specific handler trait for making HandlerFn dyn-compatible
#[async_trait]
pub(super) trait WSDynHandler: Send + Sync + 'static {
    async fn call(&self, data: Value, data_type: DataType, http: HttpType);
}

#[async_trait]
impl<F, T> WSDynHandler for WSHandler<F, T>
where
    F: HandlerFn<(T, DataType, HttpType), ()> + Send + Sync + Copy + 'static,
    T: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    async fn call(&self, data: Value, data_type: DataType, http: HttpType) {
        let json: T = match serde_json::from_value(data) {
            Ok(j) => j,
            Err(e) => {
                error!("Error deserializing WebSocket data: {:?}", e);
                return;
            }
        };
        self.callback.call((json, data_type, http)).await;
    }
}

pub(super) struct WSHandlerBuilder<T>
where
    T: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    _type: std::marker::PhantomData<T>,
}

impl<T> WSHandlerBuilder<T>
where
    T: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    pub fn build<F>(callback: F) -> WSHandler<F, T>
    where
        F: HandlerFn<(T, DataType, HttpType), ()> + Send + Sync + Copy + 'static,
    {
        WSHandler {
            callback,
            _type: std::marker::PhantomData,
        }
    }
}
