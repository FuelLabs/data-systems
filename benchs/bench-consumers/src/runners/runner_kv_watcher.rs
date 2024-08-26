use anyhow::Result;
use fuel_core_types::blockchain::block::Block;
use futures::StreamExt;
use nats_publisher::utils::nats::NatsHelper;

use super::benchmark_results::BenchmarkResult;

#[allow(dead_code)]
pub async fn run_watch_kv_blocks(
    nats: &NatsHelper,
    limit: usize,
) -> Result<()> {
    let mut result =
        BenchmarkResult::new("KV Blocks Watcher".to_string(), limit);
    let mut watch = nats.kv_blocks.watch_all().await?;

    while let Some(message) = watch.next().await {
        let item = message?;
        match nats
            .data_parser()
            .from_nats_message::<Block>(item.value.to_vec())
            .await
        {
            Err(_) => result.increment_error_count(),
            Ok(decoded) => {
                result
                    .add_publish_time(decoded.ts_as_millis())
                    .increment_message_count();
                if result.is_complete() {
                    result.finalize();
                    println!("{}", result);
                    break;
                }
            }
        }
    }

    Ok(())
}
