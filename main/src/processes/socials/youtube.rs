use std::collections::HashMap;

use framework::{build_process, processes::ProcessLoop};
use serenity::async_trait;

build_process!(YoutubeProcess, HashMap<String, String>);

#[async_trait]
impl ProcessLoop for YoutubeProcess {
    async fn process(&self, _http: utils::HttpType, _data: utils::DataType) {
        loop {
            // Your YouTube processing logic here
        }
    }
}
