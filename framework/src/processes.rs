use std::{any::Any, sync::Arc};

use downcast_rs::{DowncastSync, impl_downcast};
use serenity::{
    all::Context,
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};
use utils::{DataType, HttpType, Pointer, info};

use crate::extractors::{ContextExtractor, Extractor};

type DefaultStorage = dyn ProcessLoop + Send + Sync;

#[macro_export]
macro_rules! build_process {
    ($name:ident, $ty:ty) => {
        #[derive(Default)]
        pub struct $name(pub tokio::sync::RwLock<$ty>);

        impl serenity::prelude::TypeMapKey for $name {
            type Value = std::sync::Arc<$name>;
        }
    };
}

#[derive(Default)]
pub struct ProcessManager {
    process_vec: Vec<Arc<DefaultStorage>>,
    process_map: TypeMap,
}
impl ProcessManager {
    pub fn init_loop(&self, http: &HttpType, data: &DataType) {
        // let data = self.data.clone();
        info!("(processes) Starting process loop");
        for p in self.process_vec.iter() {
            let process_struct = p.clone();
            let http = http.clone();
            let data = data.clone();
            tokio::spawn(async move {
                process_struct.process(http.clone(), data.clone()).await;
            });
        }
    }

    pub fn register_process<P>(&mut self, process: P)
    where
        P: ProcessLoop + 'static,
        P: TypeMapKey<Value = Arc<P>>,
    {
        let process_ptr = Arc::new(process);
        self.process_vec.push(process_ptr.clone());
        self.process_map.insert::<P>(process_ptr);
    }

    pub fn get<P>(&self) -> Option<Arc<P>>
    where
        P: ProcessLoop + Send + Sync + 'static,
        P: TypeMapKey<Value = Arc<P>>,
    {
        self.process_map.get::<P>().cloned()
    }
}

impl TypeMapKey for ProcessManager {
    type Value = Arc<ProcessManager>;
}

#[async_trait]
impl ContextExtractor for Arc<ProcessManager> {
    async fn extract_context(ctx: &Context) -> Option<Self> {
        ctx.data.read().await.get::<ProcessManager>().cloned()
    }
}

#[async_trait]
impl<T> Extractor<T> for Arc<ProcessManager>
where
    T: Send + Sync + 'static,
{
    async fn extract(ctx: &Context, _: &T, _: &Pointer<utils::Parser>) -> Option<Self> {
        Arc::<ProcessManager>::extract_context(ctx).await
    }
}

#[async_trait]
pub trait ProcessLoop: Any + Send + Sync + DowncastSync {
    async fn process(&self, http: utils::HttpType, data: utils::DataType);
}

impl_downcast!(sync ProcessLoop);
