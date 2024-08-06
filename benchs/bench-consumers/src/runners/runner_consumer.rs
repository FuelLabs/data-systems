use std::time::Duration;

use anyhow::Result;
use async_nats::jetstream::consumer::AckPolicy;
pub use async_nats::jetstream::consumer::{
    pull::Config as PullConsumerConfig,
    DeliverPolicy,
};
use fuel_core_types::blockchain::block::Block;
use futures_util::StreamExt;
use nats_publisher::utils::{nats::NatsHelper, payload::NatsPayload};
use tokio::time::sleep;

use super::benchmark_results::BenchmarkResult;

pub async fn run_blocks_consumer(
    nats: &NatsHelper,
    limit: usize,
) -> Result<()> {
    let mut result = BenchmarkResult::new(
        "Blocks Consumer (Ephemeral + AckNone)".into(),
        limit,
    );

    let consumer = nats
        .stream_blocks
        .create_consumer(PullConsumerConfig {
            deliver_policy: DeliverPolicy::New,
            ack_policy: AckPolicy::None,
            ..Default::default()
        })
        .await?;

    let mut messages = consumer.messages().await?;
    while let Some(message) = messages.next().await {
        let msg = message?;
        match NatsPayload::<Block>::from_slice(&msg.payload) {
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
