use anyhow::Result;
use nats_publisher::utils::nats::NatsHelper;
use tokio::try_join;

static MSGS_LIMIT: usize = 5000;

use super::{
    runner_consumer::run_blocks_consumer,
    runner_kv_watcher::run_watch_kv_blocks,
    runner_subscription::run_subscriptions,
};

#[allow(dead_code)]
pub async fn run_all_benchmarks() -> Result<()> {
    let nats = NatsHelper::connect().await?;
    let _ = try_join!(
        run_subscriptions(&nats, MSGS_LIMIT),
        run_watch_kv_blocks(&nats, MSGS_LIMIT),
        run_blocks_consumer(&nats, MSGS_LIMIT),
    );

    Ok(())
}
