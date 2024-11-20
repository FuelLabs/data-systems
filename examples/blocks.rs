// Copyright 2024 Fuel Labs <contact@fuel.sh>
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use fuel_streams::prelude::*;
use futures::StreamExt;

/// The URL of the Fuel streaming service.
const FUEL_STREAMING_SERVICE_URL: &str = "nats://fuel-streaming.testnet:4222";

// This example demonstrates how to use the fuel-streams library to stream
// blocks from a Fuel network. It connects to a streaming service,
// subscribes to a block stream, and prints incoming blocks.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let client = Client::connect(FUEL_STREAMING_SERVICE_URL).await?;

    // Create a new stream for blocks
    let stream =
        fuel_streams::Stream::<fuel_streams_types::Block>::new(&client).await;

    // Configure the stream to start from the last published block
    let config = StreamConfig {
        deliver_policy: DeliverPolicy::Last,
    };

    // Subscribe to the block stream with the specified configuration
    let mut sub = stream.subscribe_with_config(config).await?;

    println!("Listening for blocks...");

    // Process incoming blocks
    while let Some(bytes) = sub.next().await {
        let message = bytes?;
        let decoded_msg =
            fuel_streams_types::Block::decode_raw(message.payload.to_vec())
                .await;
        let tx_subject = decoded_msg.subject;
        let tx_published_at = decoded_msg.timestamp;

        println!(
            "Received block:\n  Subject: {}\n  Published at: {}\n  Block: {:?}\n",
            tx_subject, tx_published_at, decoded_msg.payload
        );
    }

    Ok(())
}
