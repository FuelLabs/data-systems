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
    prelude::*,
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
            .with_block_height(Some(5.into()))
            .with_kind(Some(TransactionKind::Mint));
        stream_transactions(&txs_client, Some(filter))
            .await
            .unwrap();
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
