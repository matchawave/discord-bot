use chrono::DateTime;
use framework::{build_process, processes::ProcessLoop};

use serenity::async_trait;

// build_process!(BirthdayProcess, );

// #[async_trait]
// impl ProcessLoop for BirthdayProcess {
//     async fn process(&self, _: utils::HttpType, _data: utils::DataType) {
//         loop {
//             tokio::time::sleep(std::time::Duration::from_secs(10)).await;
//         }
//     }
// }
