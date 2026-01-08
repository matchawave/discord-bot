use framework::{
    data::{Cooldowns, Ephemerals},
    processes::ProcessManager,
    websocket::WebSocketProcessor,
};
use serenity::Client;

use crate::websocket::SocketReceiveEvent;

mod backend;
mod socials;

use backend::ShardUpdater;
use socials::YoutubeProcess;

pub async fn start_background_processes(
    client: &Client,
    websocket: WebSocketProcessor<SocketReceiveEvent>,
) {
    let mut manager = ProcessManager::new(client);
    manager.register_process(websocket);
    manager.register_process(Cooldowns::default());
    manager.register_process(Ephemerals::default());
    manager.register_process(YoutubeProcess::default());
    manager.register_process(ShardUpdater::default());
    // manager.register_process(process);
    manager.init_loop().await;
    client
        .data
        .write()
        .await
        .insert::<ProcessManager>(manager.into());
}
