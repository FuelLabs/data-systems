use std::time::Duration;

use anyhow::Result;
use fuel_core_types::blockchain::block::Block;
use futures_util::StreamExt;
use tokio::time::timeout;

use super::benchmark_results::BenchmarkResult;
use crate::utils::nats::NatsHelper;

#[allow(dead_code)]
pub async fn run_watch_kv_blocks() -> Result<BenchmarkResult> {
    let nats_helper = NatsHelper::connect().await?;
    let mut result = BenchmarkResult::new("KV Blocks Watcher".to_string());
    let mut watch = nats_helper.kv_blocks.watch("blocks.>").await?;

    while !result.is_complete() {
        match timeout(Duration::from_secs(5), watch.next()).await {
            Ok(Some(entry)) => {
                let item = entry?;
                match bincode::deserialize::<Block>(&item.value) {
                    Ok(_) => result.increment_message_count(),
                    Err(_) => result.increment_error_count(),
                }
            }
            Ok(None) => break,
            Err(_) => continue,
        }
    }

    result.finalize();
    Ok(result)
}
