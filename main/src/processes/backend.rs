use chrono::DateTime;
use framework::{
    ShardData, build_process, extractors::ShardManagerContainer, processes::ProcessLoop,
    websocket::WebSocketWriter,
};
use serde::Serialize;
use serenity::{all::ConnectionStage, async_trait};

use crate::{data::member_list::MemberList, websocket::SocketSendEvent};

build_process!(ShardUpdater, DateTime<chrono::Utc>);

#[async_trait]
impl ProcessLoop for ShardUpdater {
    async fn process(&self, _: utils::HttpType, data: utils::DataType) {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }
}
