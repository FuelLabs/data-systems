use std::sync::Arc;

use fuel_streams_core::prelude::*;
use tokio::task::JoinHandle;

use crate::packets::{PublishError, PublishOpts, PublishPacket};

pub fn publish_task(
    block: &Block<Transaction>,
    stream: Arc<Stream<Block>>,
    opts: &Arc<PublishOpts>,
) -> JoinHandle<Result<(), PublishError>> {
    let block_height = block.header().consensus().height.into();
    let block_producer = (*opts.block_producer).clone();
    let packet = PublishPacket::new(
        block,
        BlocksSubject {
            height: Some(block_height),
            producer: Some(block_producer),
        }
        .arc(),
        BlocksSubject::WILDCARD,
    );

    packet.publish(Arc::clone(&stream), opts)
}
