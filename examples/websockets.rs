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

use std::time::Duration;

use fuel_streams::{
    blocks::BlocksSubject,
    subjects::SubjectBuildable,
    types::FuelNetwork,
};
use fuel_streams_ws::{
    client::WebSocketClient,
    server::ws::models::DeliverPolicy,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client =
        WebSocketClient::new(FuelNetwork::Local, "admin", "admin").await?;

    client.connect().await?;

    let subject = BlocksSubject::new();
    let deliver_policy = DeliverPolicy::New;
    // .with_producer(Some(Address::zeroed()))
    // .with_height(Some(183603.into()));

    println!("Subscribing to subject {:?} ...", subject);
    client.subscribe(subject.clone(), deliver_policy).await?;

    let mut receiver = client.listen().await?;

    tokio::spawn(async move {
        while let Some(_message) = receiver.recv().await {
            // println!("Received: {:?}", message);
        }
    });

    tokio::time::sleep(Duration::from_secs(15)).await;

    println!("Unsubscribing to subject {:?} ...", subject);
    client.unsubscribe(subject, deliver_policy).await?;

    Ok(())
}
