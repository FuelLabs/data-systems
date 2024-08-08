use anyhow::Result;
use fuel_data_parser::{CompressionLevel, CompressionType, SerializationType};
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
    let use_nats_compression = false; // adjust as needed
    let mut nats = NatsHelper::connect(use_nats_compression).await?;
    nats.data_parser_mut()
        .set_serialization_type(SerializationType::Postcard); // adjust as needed
    nats.data_parser_mut()
        .set_compression_type(CompressionType::Zlib); // adjust as needed
    nats.data_parser_mut()
        .set_compression_level(CompressionLevel::Fastest); // adjust as needed

    let _ = try_join!(
        run_subscriptions(&nats, MSGS_LIMIT),
        run_watch_kv_blocks(&nats, MSGS_LIMIT),
        run_blocks_consumer(&nats, MSGS_LIMIT),
    );

    Ok(())
}
