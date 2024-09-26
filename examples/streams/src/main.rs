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

use fuel_core_types::fuel_tx::{ContractId, Receipt};
use fuel_streams::{
    blocks::BlocksSubject,
    core::prelude::SubjectBuildable,
    prelude::*,
    receipts::{
        ReceiptsBurnSubject,
        ReceiptsCallSubject,
        ReceiptsLogDataSubject,
        ReceiptsLogSubject,
        ReceiptsMintSubject,
        ReceiptsPanicSubject,
        ReceiptsReturnDataSubject,
        ReceiptsReturnSubject,
        ReceiptsRevertSubject,
        ReceiptsTransferOutSubject,
        ReceiptsTransferSubject,
    },
    transactions::TransactionsSubject,
};
use futures::{future::try_join_all, StreamExt};

const FUEL_STREAMING_SERVICE_URL: &str = "nats:://fuel-streaming.testnet:4222";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // initialize a client
    let client = Client::connect(FUEL_STREAMING_SERVICE_URL).await?;

    let mut handles = vec![];

    // stream blocks
    let stream_client = client.clone();
    handles.push(tokio::spawn(async move {
        stream_blocks(&stream_client, None).await.unwrap();
    }));

    // stream blocks with filter
    let stream_client = client.clone();
    handles.push(tokio::spawn(async move {
        let filter = Filter::<BlocksSubject>::build()
            .with_producer(Some(Address::zeroed()))
            .with_height(Some(5.into()));
        stream_blocks(&stream_client, Some(filter)).await.unwrap();
    }));

    // stream transactions
    let txs_client = client.clone();
    handles.push(tokio::spawn(async move {
        stream_transactions(&txs_client, None).await.unwrap();
    }));

    // stream transactions with filter
    let txs_client = client.clone();
    handles.push(tokio::spawn(async move {
        let filter = Filter::<TransactionsSubject>::build()
            .with_height(Some(5.into()))
            .with_kind(Some(TransactionKind::Mint));
        stream_transactions(&txs_client, Some(filter))
            .await
            .unwrap();
    }));

    // stream contract receipts
    handles.push(tokio::spawn({
        let contract_client = client.clone();
        // Replace with an actual contract ID
        let contract_id = ContractId::from([0u8; 32]);
        async move {
            stream_contract(&contract_client, contract_id)
                .await
                .unwrap();
        }
    }));

    // await all handles
    try_join_all(handles).await?;

    Ok(())
}

async fn stream_blocks(
    client: &Client,
    filter: Option<BlocksSubject>,
) -> anyhow::Result<()> {
    let mut block_stream = fuel_streams::Stream::<Block>::new(client).await;

    let mut sub = match filter {
        Some(filter) => block_stream.with_filter(filter).subscribe().await?,
        None => block_stream.subscribe().await?,
    };
    while let Some(bytes) = sub.next().await {
        let decoded_msg = Block::decode_raw(bytes.unwrap()).await;
        let block_height = *decoded_msg.payload.header().consensus().height;
        let block_subject = decoded_msg.subject;
        let block_published_at = decoded_msg.timestamp;
        println!(
            "Received block: height={}, subject={}, published_at={}",
            block_height, block_subject, block_published_at
        )
    }

    Ok(())
}

async fn stream_transactions(
    client: &Client,
    filter: Option<TransactionsSubject>,
) -> anyhow::Result<()> {
    let mut txs_stream = fuel_streams::Stream::<Transaction>::new(client).await;

    // here we apply a config to the streaming to start getting only from the last published transaction onwards
    let config = StreamConfig {
        deliver_policy: DeliverPolicy::Last,
    };

    let mut sub = match filter {
        Some(filter) => {
            txs_stream
                .with_filter(filter)
                .subscribe_with_config(config)
                .await?
        }
        None => txs_stream.subscribe_with_config(config).await?,
    };

    while let Some(bytes) = sub.next().await {
        let message = bytes?;
        let decoded_msg =
            Transaction::decode_raw(message.payload.to_vec()).await;
        let tx = decoded_msg.payload;
        let tx_subject = decoded_msg.subject;
        let tx_published_at = decoded_msg.timestamp;
        println!(
            "Received transaction: data={:?}, subject={}, published_at={}",
            tx, tx_subject, tx_published_at
        )
    }
    Ok(())
}

/// Subscribes to receipts related to a specific contract, effectively listening to contract events.
///
/// This function creates a stream that subscribes to various types of receipts (except ScriptResult
/// and MessageOut) that are associated with the specified contract ID. It's a way to monitor
/// contract-related events such as calls, returns, logs, transfers, mints, and burns.
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
/// Returns a Result which is Ok if the streaming completes successfully, or an Error if there are any issues.
async fn stream_contract(
    client: &Client,
    contract_id: ContractId,
) -> anyhow::Result<()> {
    let mut receipt_stream = fuel_streams::Stream::<Receipt>::new(client).await;

    receipt_stream.with_filter(
        ReceiptsBurnSubject::new().with_contract_id(Some(contract_id.into())),
    );
    receipt_stream.with_filter(
        ReceiptsCallSubject::new().with_from(Some(contract_id.into())),
    );
    receipt_stream.with_filter(
        ReceiptsReturnSubject::new().with_id(Some(contract_id.into())),
    );
    receipt_stream.with_filter(
        ReceiptsReturnDataSubject::new().with_id(Some(contract_id.into())),
    );
    receipt_stream.with_filter(
        ReceiptsPanicSubject::new().with_id(Some(contract_id.into())),
    );
    receipt_stream.with_filter(
        ReceiptsRevertSubject::new().with_id(Some(contract_id.into())),
    );
    receipt_stream.with_filter(
        ReceiptsLogSubject::new().with_id(Some(contract_id.into())),
    );
    receipt_stream.with_filter(
        ReceiptsLogDataSubject::new().with_id(Some(contract_id.into())),
    );
    receipt_stream.with_filter(
        ReceiptsTransferSubject::new().with_from(Some(contract_id.into())),
    );
    receipt_stream.with_filter(
        ReceiptsTransferOutSubject::new().with_from(Some(contract_id.into())),
    );
    receipt_stream.with_filter(
        ReceiptsMintSubject::new().with_contract_id(Some(contract_id.into())),
    );

    let mut sub = receipt_stream.subscribe().await?;

    while let Some(bytes) = sub.next().await {
        let decoded_msg = Receipt::decode_raw(bytes.unwrap().to_vec()).await;
        let receipt = decoded_msg.payload;

        // Check if the receipt has a contract_id and if it matches our target
        if let Some(receipt_contract_id) = receipt.contract_id() {
            if *receipt_contract_id == contract_id {
                let receipt_subject = decoded_msg.subject;
                let receipt_published_at = decoded_msg.timestamp;
                println!(
                    "Received contract receipt: data={:?}, subject={}, published_at={}",
                    receipt, receipt_subject, receipt_published_at
                );
            }
        }
    }

    Ok(())
}
