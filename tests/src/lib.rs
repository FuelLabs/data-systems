use std::time::Duration;

use fuel_streams_core::{
    nats::NatsClient,
    prelude::*,
    types::{Block, Transaction},
    Stream,
};
use tokio::task::JoinHandle;

type PublishedBlocksResult =
    BoxedResult<(Vec<(BlocksSubject, Block)>, JoinHandle<()>)>;
type PublishedTxsResult =
    BoxedResult<(Vec<(TransactionsSubject, Transaction)>, JoinHandle<()>)>;

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

pub fn publish_items<T: Streamable + 'static>(
    stream: &Stream<T>,
    items: Vec<(impl IntoSubject + Clone + 'static, T)>,
) -> JoinHandle<()> {
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
    })
}

pub fn publish_blocks(
    stream: &Stream<Block>,
    producer: Option<Address>,
    use_height: Option<u32>,
) -> PublishedBlocksResult {
    let mut items = Vec::new();
    for i in 0..10 {
        let block_item = MockBlock::build(use_height.unwrap_or(i));
        let subject = BlocksSubject::build(
            producer.clone(),
            Some((use_height.unwrap_or(i)).into()),
        );
        items.push((subject, block_item));
    }

    let join_handle = publish_items::<Block>(stream, items.clone());

    Ok((items, join_handle))
}

pub fn publish_transactions(
    stream: &Stream<Transaction>,
    mock_block: &Block,
    use_index: Option<u32>,
) -> PublishedTxsResult {
    let mut items = Vec::new();
    for i in 0..10 {
        let tx = MockTransaction::build();
        let subject = TransactionsSubject::from(&tx)
            .with_block_height(Some(mock_block.clone().into()))
            .with_index(Some(use_index.unwrap_or(i) as usize))
            .with_status(Some(TransactionStatus::Success));
        items.push((subject, tx));
    }

    let join_handle = publish_items::<Transaction>(stream, items.clone());

    Ok((items, join_handle))
}
