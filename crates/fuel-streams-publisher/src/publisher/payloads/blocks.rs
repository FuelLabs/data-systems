use std::sync::Arc;

use fuel_streams_core::prelude::*;
use tokio::task::JoinHandle;

use crate::{publish, PublishOpts};

pub fn publish_task(
    block: &FuelCoreBlock<FuelCoreTransaction>,
    stream: Arc<Stream<Block>>,
    opts: &Arc<PublishOpts>,
    transaction_ids: Vec<Bytes32>,
) -> JoinHandle<anyhow::Result<()>> {
    let block_height = (*opts.block_height).clone();
    let block_producer = (*opts.block_producer).clone();
    let consensus = (*opts.consensus).clone();

    let block = Block::new(block, consensus, transaction_ids);
    let packet = PublishPacket::new(
        block,
        BlocksSubject {
            height: Some(block_height),
            producer: Some(block_producer),
        }
        .arc(),
    );

    publish(&packet, stream, opts)
}
