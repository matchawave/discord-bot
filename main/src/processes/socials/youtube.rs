use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use framework::{
    build_process,
    extractors::ContextExtractor,
    processes::{ProcessLoop, ProcessManager},
};
use serenity::{all::ShardId, async_trait};

build_process!(YoutubeProcess, HashMap<String, String>);

#[async_trait]
impl ProcessLoop for YoutubeProcess {
    async fn process(&self, http: utils::Http) {
        // Your YouTube processing logic here
    }
}
