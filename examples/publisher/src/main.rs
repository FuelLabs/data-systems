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

use std::sync::Arc;

use fuel_core::service::Config;
use fuel_core_bin::FuelService;
use fuel_streams::prelude::*;
use fuel_streams_publisher::{metrics::PublisherMetrics, Streams};

const FUEL_STREAMING_SERVICE_URL: &str = "nats:://localhost:4222";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // initialize a client
    assert!(
        dotenvy::var("NATS_ADMIN_PASS").is_ok(),
        "NATS_ADMIN_PASS must be set"
    );

    let opts = NatsClientOpts::admin_opts(FUEL_STREAMING_SERVICE_URL)
        .with_rdn_namespace()
        .with_timeout(5);
    let client = Client::with_opts(&opts).await?;
    assert!(client.conn.is_connected());

    // create local fuel core service
    let fuel_core = FuelService::new_node(Config::local_node()).await?;
    let fuel_core = Arc::new(fuel_core);

    // start fuel core
    fuel_core.start_and_await().await?;

    // create the publisher
    let publisher = fuel_streams_publisher::Publisher::new(
        Arc::clone(&fuel_core),
        FUEL_STREAMING_SERVICE_URL,
        Arc::new(PublisherMetrics::new(None)?),
        Arc::new(Streams::new(&client.conn.clone()).await),
    )
    .await?;

    // run publisher in the background
    publisher.run().await?;

    Ok(())
}
