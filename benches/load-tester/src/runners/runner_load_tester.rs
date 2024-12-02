use anyhow::Result;
pub use async_nats::jetstream::consumer::DeliverPolicy;
use fuel_streams::{client::Client, StreamConfig};
use fuel_streams_core::prelude::*;
use futures::StreamExt;

use super::benchmark_results::BenchmarkResult;

pub async fn run_blocks_consumer(client: &Client) -> Result<()> {
    let mut result = BenchmarkResult::new("Blocks Consumer".into());

    // Create a new stream for blocks
    let stream = fuel_streams::Stream::<Block>::new(client).await;

    // Configure the stream to start from the last published block
    let config = StreamConfig {
        deliver_policy: DeliverPolicy::Last,
    };

    // Subscribe to the block stream with the specified configuration
    let mut sub = stream.subscribe_with_config(config).await?;

    // Process incoming blocks
    while let Some(bytes) = sub.next().await {
        match bytes.as_ref() {
            Err(_) => result.increment_error_count(),
            Ok(message) => {
                let decoded_msg =
                    Block::decode_raw(message.payload.to_vec()).await;
                let tx_subject = decoded_msg.subject.clone();
                let tx_published_at = decoded_msg.timestamp.clone();
                let ts_millis = decoded_msg.ts_as_millis();

                println!(
                    "Received block:\n  Subject: {}\n  Published at: {}\n  Block: {:?}\n",
                    tx_subject, tx_published_at, decoded_msg.payload
                );

                result.add_publish_time(ts_millis).increment_message_count();
            }
        }
    }

    Ok(())
}
