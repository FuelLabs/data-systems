use fuel_streams_core::{
    blocks::BlocksSubject,
    types::{Block, Transaction},
    Stream,
};
use tracing::info;

pub async fn publish(
    blocks_stream: &Stream<Block>,
    block: &Block<Transaction>,
) -> anyhow::Result<()> {
    let height = block.header().consensus().height;
    let block_subject: BlocksSubject = block.into();

    // Publish the block.
    info!("NATS Publisher: Publishing Block #{height}");

    blocks_stream.publish(&block_subject, block).await?;

    Ok(())
}
