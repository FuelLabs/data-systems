use std::sync::Arc;

use anyhow::Result;
pub use async_nats::jetstream::consumer::DeliverPolicy;
use fuel_streams::{client::Client, StreamConfig};
use fuel_streams_core::prelude::*;
use futures::StreamExt;

use super::results::LoadTestTracker;

pub async fn run_blocks_consumer(
    client: &Client,
    load_test_tracker: Arc<LoadTestTracker>,
) -> Result<()> {
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
            Err(_) => load_test_tracker.increment_error_count(),
            Ok(message) => {
                load_test_tracker.increment_message_count();
                let decoded_msg =
                    Block::decode_raw(message.payload.to_vec()).await;
                let ts_millis = decoded_msg.ts_as_millis();

                // let tx_subject = decoded_msg.subject.clone();
                // let tx_published_at = decoded_msg.timestamp.clone();
                // println!(
                //     "Received block:\n  Subject: {}\n  Published at: {}\n  Block: {:?}\n",
                //     tx_subject, tx_published_at, decoded_msg.payload
                // );

                load_test_tracker
                    .add_publish_time(ts_millis)
                    .increment_message_count();
            }
        }
    }

    Ok(())
}
