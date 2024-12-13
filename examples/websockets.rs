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

use fuel_streams::{
    blocks::BlocksSubject,
    subjects::SubjectBuildable,
    types::{DeliverPolicy, FuelNetwork},
};
use fuel_streams_ws::client::WebSocketClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client =
        WebSocketClient::new(FuelNetwork::Local, "admin", "admin").await?;

    client.connect()?;

    let subject = BlocksSubject::new();
    // .with_producer(Some(Address::zeroed()))
    // .with_height(Some(23.into()));

    client.subscribe(subject.clone(), DeliverPolicy::All)?;

    let mut receiver = client.listen()?;

    tokio::spawn(async move {
        while let Some(_message) = receiver.recv().await {
            // println!("Received: {:?}", message);
        }
    })
    .await?;

    Ok(())
}
