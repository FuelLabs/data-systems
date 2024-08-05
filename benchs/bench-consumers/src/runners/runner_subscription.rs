use std::time::Duration;

use anyhow::Result;
use futures_util::StreamExt;
use tokio::time::timeout;

use super::benchmark_results::BenchmarkResult;
use crate::utils::nats::connect;

#[allow(dead_code)]
pub async fn run_subscriptions() -> Result<BenchmarkResult> {
    let client = connect().await?;
    let mut result = BenchmarkResult::new("Subscriptions".to_string());
    let mut subscriber = client.subscribe("blocks.>").await?;

    while !result.is_complete() {
        match timeout(Duration::from_secs(5), subscriber.next()).await {
            Ok(Some(_)) => result.increment_message_count(),
            Ok(None) => break,
            Err(_) => continue,
        }
    }

    result.finalize();
    Ok(result)
}
