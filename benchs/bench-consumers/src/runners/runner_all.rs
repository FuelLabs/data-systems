use anyhow::Result;

use super::{
    runner_consumer::*,
    runner_kv_watcher::run_watch_kv_blocks,
    runner_subscription::run_subscriptions,
};

#[allow(dead_code)]
pub async fn run_all_benchmarks() -> Result<()> {
    let results = vec![
        run_consume_blocks_encoded_durable().await?,
        run_consume_blocks_encoded_ack_none().await?,
        run_watch_kv_blocks().await?,
        run_subscriptions().await?,
    ];

    for result in results {
        result.print_result();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_all_benchmarks() {
        run_all_benchmarks().await.unwrap();
    }
}
