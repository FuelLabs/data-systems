use std::sync::Arc;

use fuel_streams_core::prelude::*;
use tokio::task::JoinHandle;

use crate::packets::{PublishOpts, PublishPacket};

pub fn publish_task(
    block: &FuelCoreBlock<Transaction>,
    stream: Arc<Stream<fuel_streams_types::Block>>,
    opts: &Arc<PublishOpts>,
) -> JoinHandle<anyhow::Result<()>> {
    let block_height = block.header().consensus().height;
    let block_producer = (*opts.block_producer).clone();
    let consensus = (*opts.consensus).clone();

    let block = fuel_streams_types::Block::new(block, consensus);
    let packet = PublishPacket::new(
        &block,
        BlocksSubject {
            height: Some(block_height.into()),
            producer: Some(block_producer),
        }
        .arc(),
    );

    packet.publish(Arc::clone(&stream), opts)
}
