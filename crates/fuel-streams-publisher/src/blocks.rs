use fuel_streams_core::{
    blocks::BlocksSubject,
    types::{Block, BlockHeight, Transaction},
    Stream,
};
use tracing::info;

pub async fn publish(
    block_height: &BlockHeight,
    blocks_stream: &Stream<Block>,
    block: &Block<Transaction>,
) -> anyhow::Result<()> {
    let block_subject: BlocksSubject = block.into();

    info!("NATS Publisher: Publishing Block #{block_height}");

    blocks_stream.publish(&block_subject, block).await?;

    Ok(())
}
