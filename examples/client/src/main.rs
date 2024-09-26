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

const FUEL_STREAMING_SERVICE_URL: &str = "nats:://fuel-streaming.testnet:4222";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // initialize a default client with all default settings
    let client = Client::connect(FUEL_STREAMING_SERVICE_URL).await?;
    assert!(client.conn.is_connected());

    // initialize a default client with some user options
    let opts = NatsClientOpts::default_opts(FUEL_STREAMING_SERVICE_URL)
        .with_namespace("fuel")
        .with_timeout(5);
    let client = Client::with_opts(&opts).await?;
    assert!(client.conn.is_connected());

    // initialize an admin client with admin settings
    // NOTE: NATS_ADMIN_PASS environment variable is needed to be set here
    assert!(
        dotenvy::var("NATS_ADMIN_PASS").is_ok(),
        "NATS_ADMIN_PASS must be set"
    );

    let opts = NatsClientOpts::admin_opts(FUEL_STREAMING_SERVICE_URL)
        .with_namespace("fuel")
        .with_timeout(5);
    let client = Client::with_opts(&opts).await?;
    assert!(client.conn.is_connected());

    Ok(())
}
