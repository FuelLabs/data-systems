use std::time::Duration;

use fuel_streams_core::prelude::*;

pub async fn server_setup() -> BoxedResult<(NatsClient, ConnStreams)> {
    let opts = ClientOpts::admin_opts(NATS_URL).with_rdn_namespace();
    let client = NatsClient::connect(opts).await?;
    let streams = ConnStreams::new(&client).await?;
    Ok((client, streams))
}

pub fn publish_blocks(
    stream: &Streamer<Block>,
    producer: Option<String>,
) -> BoxedResult<Vec<(BlocksSubject, Block)>> {
    let mut items = Vec::new();
    for i in 0..10 {
        let block_item = MockBlock::build(i);
        let subject = BlocksSubject::build(producer.clone(), Some(i.into()));
        items.push((subject, block_item));
    }

    tokio::task::spawn({
        let stream = stream.clone();
        let items = items.clone();
        async move {
            for item in items {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let payload = item.1.clone();
                stream.publish(&item.0.parse(), &payload).await.unwrap();
            }
        }
    });

    Ok(items)
}

pub fn publish_transactions(
    stream: &Streamer<Transaction>,
    mock_block: &Block,
) -> BoxedResult<Vec<(TransactionsSubject, Transaction)>> {
    let mut items = Vec::new();
    for i in 0..10 {
        let tx = MockTransaction::build();
        let subject = TransactionsSubject::from(tx.clone())
            .with_height(Some(mock_block.clone().into()))
            .with_tx_index(Some(i))
            .with_status(Some(TransactionStatus::Success));
        items.push((subject, tx));
    }

    tokio::task::spawn({
        let stream = stream.clone();
        let items = items.clone();
        async move {
            for item in items {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let payload = item.1.clone();
                stream.publish(&item.0.parse(), &payload).await.unwrap();
            }
        }
    });

    Ok(items)
}
