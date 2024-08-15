use fuel_streams_core::prelude::*;
use tracing::info;

pub async fn publish(
    block_height: &BlockHeight,
    blocks_stream: &Streamer<Block>,
    block: &Block<Transaction>,
) -> anyhow::Result<()> {
    let block_subject: BlocksSubject = block.into();

    info!("NATS Publisher: Publishing Block #{block_height}");

    blocks_stream.publish(&block_subject, block).await?;

    Ok(())
}
