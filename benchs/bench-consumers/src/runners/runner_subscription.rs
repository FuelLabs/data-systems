use anyhow::Result;
use fuel_core_types::blockchain::block::Block;
use futures_util::StreamExt;
use nats_publisher::utils::{nats::NatsHelper, payload::NatsPayload};

use super::benchmark_results::BenchmarkResult;

#[allow(dead_code)]
pub async fn run_subscriptions(nats: &NatsHelper, limit: usize) -> Result<()> {
    let mut result = BenchmarkResult::new("Pub/Sub".to_string(), limit);
    let mut subscriber = nats.client.subscribe("blocks.sub.*").await?;
    while let Some(message) = subscriber.next().await {
        let payload = message.payload;
        match NatsPayload::<Block>::from_slice(&payload) {
            Err(_) => result.increment_error_count(),
            Ok(decoded) => {
                result
                    .add_publish_time(decoded.timestamp)
                    .increment_message_count();
                if result.is_complete() {
                    result.finalize().print_result();
                    break;
                }
            }
        }
    }

    Ok(())
}
