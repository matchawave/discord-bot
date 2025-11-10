use framework::processes::ProcessManager;
use serenity::Client;

pub async fn start_background_processes(client: &Client, shards: usize) {
    let mut manager = ProcessManager::new(client.data.clone(), client.http.clone(), shards);
    // manager.register_process(process);
    manager.init_loop().await;
}
