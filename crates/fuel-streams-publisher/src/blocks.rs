use fuel_streams_core::{
    blocks::BlocksSubject,
    types::{Address, Block, BlockHeight, Transaction},
    Stream,
};
use fuel_streams_macros::subject::IntoSubject;
use tracing::info;

pub async fn publish(
    block_height: &BlockHeight,
    blocks_stream: &Stream<Block>,
    block: &Block<Transaction>,
    block_producer: &Address,
) -> anyhow::Result<()> {
    let block_subject: BlocksSubject = BlocksSubject::new()
        .with_height(Some(block_height.clone()))
        .with_producer(Some(block_producer.clone()));

    info!("NATS Publisher: Publishing Block #{block_height}");

    blocks_stream.publish(&block_subject, block).await?;

    Ok(())
}
