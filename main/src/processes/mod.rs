use framework::{
    data::{Cooldowns, Ephemerals},
    processes::ProcessManager,
};
use serenity::Client;

use crate::processes::socials::YoutubeProcess;

mod socials;

pub async fn start_background_processes(client: &Client) {
    let mut manager = ProcessManager::new(client);
    manager.register_process(Cooldowns::default());
    manager.register_process(Ephemerals::default());
    manager.register_process(YoutubeProcess::default());
    // manager.register_process(process);
    manager.init_loop().await;
    client
        .data
        .write()
        .await
        .insert::<ProcessManager>(std::sync::Arc::new(manager));
}
