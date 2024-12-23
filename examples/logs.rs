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
// logs from a Fuel network. It connects to a streaming service,
// subscribes to a log stream, and prints incoming logs.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Testnet).await?;
    let mut connection = client.connect().await?;

    println!("Listening for logs...");

    let subject = LogsSubject::new();
    // Subscribe to the log stream with the specified configuration
    let mut stream = connection
        .subscribe::<Log>(subject, DeliverPolicy::Last)
        .await?;

    // Process incoming logs
    while let Some(log) = stream.next().await {
        println!("Received log: {:?}", log);
    }

    Ok(())
}
