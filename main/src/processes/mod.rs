use std::sync::Arc;

use framework::{
    data::{Cooldowns, Ephemerals},
    processes::ProcessManager,
    websocket::WebSocketProcessor,
};
use serenity::prelude::TypeMap;

use crate::websocket::SocketReceiveEvent;

mod backend;
mod socials;

use backend::ShardUpdater;
use socials::YoutubeProcess;

pub async fn get_bg_process_manager(
    data: &mut TypeMap,
    websocket: WebSocketProcessor<SocketReceiveEvent>,
) -> Arc<ProcessManager> {
    let mut manager = ProcessManager::default();
    manager.register_process(websocket);
    manager.register_process(Cooldowns::default());
    manager.register_process(Ephemerals::default());
    manager.register_process(YoutubeProcess::default());
    manager.register_process(ShardUpdater::default());

    let manager = Arc::new(manager);
    data.insert::<ProcessManager>(manager.clone());
    manager
}
