use std::time::Duration;

use anyhow::Result;
pub use async_nats::jetstream::consumer::{
    pull::Config as PullConsumerConfig,
    AckPolicy,
    DeliverPolicy,
};
use fuel_core_types::blockchain::block::Block;
use futures_util::StreamExt;
use tokio::time::timeout;

use super::benchmark_results::BenchmarkResult;
use crate::utils::nats::NatsHelper;

async fn run_consume_blocks(
    name: &str,
    config: PullConsumerConfig,
) -> Result<BenchmarkResult> {
    let mut result = BenchmarkResult::new(name.to_string());
    let nats_helper = NatsHelper::connect().await?;
    let consumer = nats_helper
        .st_blocks_encoded
        .create_consumer(config)
        .await?;

    let mut messages = consumer.messages().await?;
    while !result.is_complete() {
        match timeout(Duration::from_secs(5), messages.next()).await {
            Ok(Some(message)) => {
                let msg = message?;
                if name.contains("json") {
                    match serde_json::from_slice::<Block>(&msg.payload) {
                        Ok(_) => result.increment_message_count(),
                        Err(_) => result.increment_error_count(),
                    }
                } else {
                    match bincode::deserialize::<Block>(&msg.payload) {
                        Ok(_) => result.increment_message_count(),
                        Err(_) => result.increment_error_count(),
                    }
                }
            }
            Ok(None) => break,
            Err(_) => continue,
        }
    }

    result.finalize();
    Ok(result)
}

pub async fn run_consume_blocks_encoded_durable() -> Result<BenchmarkResult> {
    run_consume_blocks(
        "Blocks Consumer (Encoded)",
        PullConsumerConfig {
            durable_name: Some("blocks_consumer_encoded".into()),
            deliver_policy: DeliverPolicy::New,
            filter_subject: "blocks.encoded.>".into(),
            ..Default::default()
        },
    )
    .await
}

pub async fn run_consume_blocks_encoded_ack_none() -> Result<BenchmarkResult> {
    run_consume_blocks(
        "Blocks Consumer (Encoded + AckNone)",
        PullConsumerConfig {
            deliver_policy: DeliverPolicy::New,
            filter_subject: "blocks.encoded.>".into(),
            ack_policy: AckPolicy::None,
            ..Default::default()
        },
    )
    .await
}
