use std::sync::Arc;

use serenity::{
    all::{Http, ShardId},
    async_trait,
    prelude::TypeMap,
};
use tokio::sync::RwLock;
use utils::{error, info};

use crate::{
    DataExtractable,
    data::{Cooldowns, Ephemerals},
};

#[macro_export]
macro_rules! build_process {
    ($name:ident, $ty:ty) => {
        use $crate::DataExtractable;

        #[derive(Clone, Default, DataExtractable)]
        pub struct $name(pub utils::Pointer<$ty>);

        impl serenity::prelude::TypeMapKey for $name {
            type Value = utils::Pointer<$ty>;
        }
    };
}

pub struct ProcessManager {
    datas: Arc<RwLock<TypeMap>>,
    http: Arc<Http>,
    shards: usize,
    processes: Vec<Arc<dyn ProcessLoop + Send + Sync>>,
}
impl ProcessManager {
    pub fn new(data: Arc<RwLock<TypeMap>>, http: Arc<Http>, shards: usize) -> Self {
        Self {
            datas: data,
            http,
            shards,
            processes: Vec::new(),
        }
    }
    pub async fn init_loop(&self) {
        for shard in 0..self.shards {
            let shard = ShardId(shard as u32);
            let Some(data) = crate::data::Data::get(&self.datas, shard).await else {
                error!("(processes) Failed to get data for shard {}", shard);
                continue;
            };

            let data = data.clone();
            let http = self.http.clone();
            let processes = Arc::new(self.processes.clone());

            tokio::spawn(Self::process_loop(shard, data, http, processes));
        }
    }

    pub fn register_process<P>(&mut self, process: P)
    where
        P: ProcessLoop + Send + Sync + 'static,
    {
        self.processes.push(Arc::new(process));
    }

    async fn process_loop(
        shard: ShardId,
        data: Arc<TypeMap>,
        http: Arc<Http>,
        processes: Arc<Vec<Arc<dyn ProcessLoop + Send + Sync>>>,
    ) {
        info!("(processes) Starting process loop for shard {}", shard);
        loop {
            if let Some(cooldowns) = Cooldowns::retrieve(&data) {
                cooldowns.process(http.clone()).await;
            }

            if let Some(ephemerals) = Ephemerals::retrieve(&data) {
                ephemerals.process(http.clone()).await;
            }

            for p in processes.iter() {
                p.process(http.clone()).await;
            }
        }
    }
}

#[async_trait]
pub trait ProcessLoop {
    async fn process(&self, http: utils::Http);
}
