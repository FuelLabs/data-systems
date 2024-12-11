use std::sync::Arc;

use anyhow::Result;
pub use async_nats::jetstream::consumer::DeliverPolicy;
use fuel_streams::{client::Client, StreamConfig};
use fuel_streams_core::prelude::*;
use futures::StreamExt;

use super::results::LoadTestTracker;

pub async fn run_streamable_consumer<S: Streamable>(
    client: &Client,
    load_test_tracker: Arc<LoadTestTracker>,
) -> Result<()> {
    // Create a new stream for blocks
    let stream = fuel_streams::Stream::<S>::new(client).await;

    // Configure the stream to start from the last published block
    let config = StreamConfig {
        deliver_policy: DeliverPolicy::Last,
    };

    // Subscribe to the block stream with the specified configuration
    let mut sub = stream.subscribe_raw_with_config(config).await?;

    // Process incoming blocks
    while let Some(bytes) = sub.next().await {
        load_test_tracker.increment_message_count();
        let decoded_msg = S::decode_raw(bytes).unwrap();

        let ts_millis = decoded_msg.ts_as_millis();
        load_test_tracker
            .add_publish_time(ts_millis)
            .increment_message_count();
    }

    Ok(())
}
