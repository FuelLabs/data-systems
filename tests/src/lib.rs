use std::time::Duration;

use fuel_streams_core::{
    nats::NatsClient,
    prelude::*,
    types::{Block, Transaction},
    Stream,
};

#[derive(Debug, Clone)]
pub struct Streams {
    pub blocks: Stream<Block>,
    pub transactions: Stream<Transaction>,
}

impl Streams {
    pub async fn new(client: &NatsClient) -> Self {
        let blocks = Stream::<Block>::get_or_init(client).await;
        let transactions = Stream::<Transaction>::get_or_init(client).await;
        Self {
            transactions,
            blocks,
        }
    }
}

pub async fn server_setup() -> BoxedResult<(NatsClient, Streams)> {
    let opts = NatsClientOpts::admin_opts(NATS_URL).with_rdn_namespace();
    let client = NatsClient::connect(&opts).await?;
    let streams = Streams::new(&client).await;
    Ok((client, streams))
}

pub fn publish_items<T: Streamable>(
    stream: &Stream<T>,
    items: Vec<(impl IntoSubject + Sync + Send + 'static, T)>,
) {
    tokio::task::spawn({
        let stream = stream.clone();
        let items = items.clone();
        async move {
            for item in items {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let payload = item.1.clone();
                let subject = item.0;
                stream.publish(&subject, &payload).await.unwrap();
            }
        }
    });
}

pub fn publish_same_blocks(
    stream: &Stream<Block>,
    producer: Option<String>,
) -> BoxedResult<Vec<(BlocksSubject, Block)>> {
    let height = 99;
    let block_item = MockBlock::build(height);
    let mut items = Vec::new();
    for _ in 0..10 {
        let subject =
            BlocksSubject::build(producer.clone(), Some(height.into()));
        items.push((subject, block_item.clone()));
    }

    publish_items::<Block>(stream, items.clone());

    Ok(items)
}

pub fn publish_blocks(
    stream: &Stream<Block>,
    producer: Option<String>,
) -> BoxedResult<Vec<(BlocksSubject, Block)>> {
    let mut items = Vec::new();
    for i in 0..10 {
        let block_item = MockBlock::build(i);
        let subject = BlocksSubject::build(producer.clone(), Some(i.into()));
        items.push((subject, block_item));
    }

    publish_items::<Block>(stream, items.clone());

    Ok(items)
}

pub fn publish_transactions(
    stream: &Stream<Transaction>,
    mock_block: &Block,
) -> BoxedResult<Vec<(TransactionsSubject, Transaction)>> {
    let mut items = Vec::new();
    for i in 0..10 {
        let tx = MockTransaction::build();
        let subject = TransactionsSubject::from(&tx)
            .with_height(Some(mock_block.clone().into()))
            .with_tx_index(Some(i))
            .with_status(Some(TransactionStatus::Success));
        items.push((subject, tx));
    }

    publish_items::<Transaction>(stream, items.clone());

    Ok(items)
}
