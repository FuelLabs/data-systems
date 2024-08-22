use anyhow::Result;
use async_nats::jetstream::consumer::AckPolicy;
pub use async_nats::jetstream::consumer::{
    pull::Config as PullConsumerConfig,
    DeliverPolicy,
};
use fuel_core_types::blockchain::block::Block;
use futures::StreamExt;
use nats_publisher::utils::nats::NatsHelper;

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
        match nats
            .data_parser()
            .from_nats_message::<Block>(msg.payload.to_vec())
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
