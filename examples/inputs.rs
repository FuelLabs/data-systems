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
// inputs from a Fuel network. It connects to a streaming service,
// subscribes to an input stream, and prints incoming inputs.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Testnet).await?;
    let mut connection = client.connect().await?;

    println!("Listening for inputs...");

    let subject = InputsCoinSubject::new();
    // Subscribe to the input stream with the specified configuration
    let mut stream = connection
        .subscribe::<Input>(subject, DeliverPolicy::Last)
        .await?;

    // Process incoming inputs
    while let Some(input) = stream.next().await {
        println!("Received input: {:?}", input);
    }

    Ok(())
}
