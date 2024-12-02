use std::sync::Arc;

use anyhow::Result;
pub use async_nats::jetstream::consumer::DeliverPolicy;
use fuel_streams::{client::Client, StreamConfig};
use fuel_streams_core::prelude::*;
use futures::StreamExt;

use super::results::LoadTestTracker;

pub async fn run_outputs_consumer(
    client: &Client,
    load_test_tracker: Arc<LoadTestTracker>,
) -> Result<()> {
    // Create a new stream for outputs
    let stream = fuel_streams::Stream::<Output>::new(client).await;

    // Configure the stream to start from the last published output
    let config = StreamConfig {
        deliver_policy: DeliverPolicy::Last,
    };

    // Subscribe to the outputs stream with the specified configuration
    let mut sub = stream.subscribe_with_config(config).await?;

    // Process incoming outputs
    while let Some(bytes) = sub.next().await {
        match bytes.as_ref() {
            Err(_) => load_test_tracker.increment_error_count(),
            Ok(message) => {
                load_test_tracker.increment_message_count();
                let decoded_msg =
                    Input::decode_raw(message.payload.to_vec()).await;
                let ts_millis = decoded_msg.ts_as_millis();

                // let tx_subject = decoded_msg.subject.clone();
                // let tx_published_at = decoded_msg.timestamp.clone();
                // println!(
                //     "Received output:\n  Subject: {}\n  Published at: {}\n  Output: {:?}\n",
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
