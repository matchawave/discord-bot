use std::sync::Arc;

use serenity::{
    all::{Http, ShardId},
    async_trait,
    prelude::TypeMap,
};
use tokio::sync::RwLock;
use utils::info;

use crate::data::{Cooldowns, Ephemerals};

#[macro_export]
macro_rules! build_process {
    ($name:ident, $ty:ty) => {
        use $crate::{DataExtractable, Extractable};

        #[derive(Clone, Default)]
        pub struct $name(pub utils::Pointer<$ty>);

        impl serenity::prelude::TypeMapKey for $name {
            type Value = utils::Pointer<$ty>;
        }

        impl $name {
            pub async fn
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
            let http = self.http.clone();
            let shard = ShardId(shard as u32);
            let data = self.datas.clone();
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
        data: Arc<RwLock<TypeMap>>,
        http: Arc<Http>,
        processes: Arc<Vec<Arc<dyn ProcessLoop + Send + Sync>>>,
    ) {
        info!("(processes) Starting process loop for shard {}", shard);
        loop {
            {
                let data_read = data.read().await;
                if let Some(cooldowns) = data_read.get::<Cooldowns>().cloned().map(Cooldowns) {
                    cooldowns.process(shard, http.clone()).await;
                }

                if let Some(ephemerals) = data_read.get::<Ephemerals>().cloned().map(Ephemerals) {
                    ephemerals.process(shard, http.clone()).await;
                }
            }

            for p in processes.iter() {
                p.process(shard, http.clone()).await;
            }
        }
    }
}

#[async_trait]
pub trait ProcessLoop {
    async fn process(&self, shard_id: ShardId, http: utils::Http);
}
