use std::sync::Arc;

use framework::{
    data::{Cooldowns, Ephemerals},
    processes::ProcessManager,
    websocket::WebSocketProcessor,
};
use serenity::prelude::TypeMap;

use crate::websocket::SocketReceiveEvent;

mod backend;
mod loader;
mod misc;
mod socials;

// use backend::ShardUpdater;
pub use loader::{AfkInstance, AfkStatus};
// use socials::YoutubeProcess;

pub async fn get_bg_process_manager(
    data: &mut TypeMap,
    websocket: WebSocketProcessor<SocketReceiveEvent>,
) -> Arc<ProcessManager> {
    let mut manager = ProcessManager::default();
    manager.register_process(websocket);
    manager.register_process(Cooldowns::default());
    manager.register_process(Ephemerals::default());
    // manager.register_process(YoutubeProcess);
    // manager.register_process(ShardUpdater);
    manager.register_process(AfkInstance::default());

    let manager = Arc::new(manager);
    data.insert::<ProcessManager>(manager.clone());
    manager
}
