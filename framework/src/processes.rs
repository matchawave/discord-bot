use std::sync::Arc;

use serenity::{
    Client,
    all::{Context, ShardId},
    async_trait,
    prelude::{TypeMap, TypeMapKey},
};
use tokio::sync::RwLock;
use utils::{DataType, Http, Pointer, info};

use crate::extractors::{ContextExtractor, Extractor};

// use crate::data::{Cooldowns, Ephemerals};

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

pub struct ProcessManager {
    data: DataType,
    http: Http,
    process_vec: Vec<Arc<dyn ProcessLoop>>,
    process_map: TypeMap,
}
impl ProcessManager {
    pub fn new(ctx: &Client) -> Self {
        Self {
            data: ctx.data.clone(),
            http: ctx.http.clone(),
            process_vec: Vec::new(),
            process_map: TypeMap::new(),
        }
    }

    pub async fn init_loop(&self) {
        let data = self.data.clone();
        info!("(processes) Starting process loop");
        for p in self.process_vec.iter() {
            let process = p.clone();
            let http = self.http.clone();
            tokio::spawn(async move {
                loop {
                    process.process(http.clone()).await;
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
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
        P: TypeMapKey<Value = Arc<P>> + Send + Sync + 'static,
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
pub trait ProcessLoop: Send + Sync {
    async fn process(&self, http: utils::Http);
}
