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

// This example demonstrates how to use the fuel-streams library to stream
// UTXOs from a Fuel network. It connects to a streaming service,
// subscribes to a UTXO stream, and prints incoming UTXOs.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let client = Client::connect(FuelNetwork::Testnet).await?;

    // Create a new stream for UTXOs
    let stream = fuel_streams::Stream::<Utxo>::new(&client).await;

    // Configure the stream to start from the last published UTXO
    let config = StreamConfig {
        deliver_policy: DeliverPolicy::Last,
    };

    // Subscribe to the UTXO stream with the specified configuration
    let mut sub = stream.subscribe_with_config(config).await?;

    println!("Listening for UTXOs...");

    // Process incoming UTXOs
    while let Some(bytes) = sub.next().await {
        let message = bytes?;
        let decoded_msg = Utxo::decode_raw(message.payload.to_vec());
        let utxo_subject = decoded_msg.subject;
        let utxo_published_at = decoded_msg.timestamp;

        println!(
            "Received UTXO:\n  Subject: {}\n  Published at: {}\n  UTXO: {:?}\n",
            utxo_subject, utxo_published_at, decoded_msg.payload
        );
    }

    Ok(())
}
