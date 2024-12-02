use anyhow::Result;
use nats_publisher::utils::nats::NatsHelper;
use tokio::try_join;

static MSGS_LIMIT: usize = 5000;

use super::runner_load_tester::run_blocks_consumer;

#[allow(dead_code)]
pub async fn run_all_benchmarks() -> Result<()> {
    let use_nats_compression = false; // adjust as needed
    let nats = NatsHelper::connect(use_nats_compression).await?;

    let _ = try_join!(run_blocks_consumer(&nats, MSGS_LIMIT),);

    Ok(())
}
