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

use anyhow::Result;
use fuel_streams::{prelude::*, receipts::*};
use futures::StreamExt;

/// The URL of the Fuel streaming service.
const FUEL_STREAMING_SERVICE_URL: &str =
    "nats://fuel-streaming-service.fuel.sh:4222";

/// The contract ID to stream the receipts for. For this example, we're using the contract ID of the https://thundernft.market/
const CONTRACT_ID: &str =
    "0x243ef4c2301f44eecbeaf1c39fee9379664b59a2e5b75317e8c7e7f26a25ed4d";

/// Subscribes to receipts related to a specific contract, effectively listening to contract events.
///
/// This function creates a stream that subscribes to various types of receipts
///
/// The function filters the receipts to ensure they match the given contract ID before processing them.
/// This approach allows for efficient monitoring of contract activities without the need to process
/// irrelevant receipts.
///
/// # Arguments
///
/// * `client` - A reference to the NATS client used for streaming.
/// * `contract_id` - The ID of the contract to monitor.
///
/// # Returns
///
/// Returns `Ok(())` if the streaming completes successfully, or an error if there are any issues.

// This example demonstrates how to use the fuel-streams library to stream
// receipts from a Fuel network. It connects to a streaming service,
// subscribes to a receipt for a given contract, and prints incoming receipts.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let client = Client::connect(FUEL_STREAMING_SERVICE_URL).await?;

    let contract_id: ContractId = CONTRACT_ID.into();

    // Create a new stream for receipts
    let mut receipt_stream =
        fuel_streams::Stream::<Receipt>::new(&client).await;

    // Use multiple filters to subscribe to different types of receipts (all receipts except
    // `ScriptResult` and `MessageOut`) that are associated with the specified contract ID.
    // It's a way to monitor all contract-related events such as calls, returns, logs, transfers,
    // mints, and burns.
    receipt_stream.with_filter(
        ReceiptsBurnSubject::default()
            .with_contract_id(Some(contract_id.clone().into())),
    );
    receipt_stream.with_filter(
        ReceiptsCallSubject::default()
            .with_from(Some(contract_id.clone().into())),
    );
    receipt_stream.with_filter(
        ReceiptsReturnSubject::default()
            .with_id(Some(contract_id.clone().into())),
    );
    receipt_stream.with_filter(
        ReceiptsReturnDataSubject::default()
            .with_id(Some(contract_id.clone().into())),
    );
    receipt_stream.with_filter(
        ReceiptsPanicSubject::default()
            .with_id(Some(contract_id.clone().into())),
    );
    receipt_stream.with_filter(
        ReceiptsRevertSubject::default()
            .with_id(Some(contract_id.clone().into())),
    );
    receipt_stream.with_filter(
        ReceiptsLogSubject::default().with_id(Some(contract_id.clone().into())),
    );
    receipt_stream.with_filter(
        ReceiptsLogDataSubject::default()
            .with_id(Some(contract_id.clone().into())),
    );
    receipt_stream.with_filter(
        ReceiptsTransferSubject::default()
            .with_from(Some(contract_id.clone().into())),
    );
    receipt_stream.with_filter(
        ReceiptsTransferOutSubject::default()
            .with_from(Some(contract_id.clone().into())),
    );
    receipt_stream.with_filter(
        ReceiptsMintSubject::default()
            .with_contract_id(Some(contract_id.clone().into())),
    );

    // Configure the stream to start from the first published receipt
    let config = StreamConfig {
        deliver_policy: DeliverPolicy::All,
    };

    // Subscribe to the receipt stream
    let mut sub = receipt_stream.subscribe_with_config(config).await?;

    println!("Listening for receipts...");

    // Process incoming receipts
    while let Some(bytes) = sub.next().await {
        let message = bytes.unwrap();
        let decoded_msg = Receipt::decode_raw(message.payload.to_vec()).await;
        let receipt = decoded_msg.payload;
        let receipt_subject = decoded_msg.subject;
        let receipt_published_at = decoded_msg.timestamp;
        println!(
            "Received receipt:\n  Subject: {}\n  Published at: {}\n  Data: {:?}\n",
            receipt_subject, receipt_published_at, receipt
        );
    }

    Ok(())
}
