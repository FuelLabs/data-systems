use std::sync::Arc;

use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::publisher::packets::{PublishError, PublishOpts, PublishPacket};

pub fn publish_tasks(
    tx_id: &Bytes32,
    stream: &Stream<Log>,
    opts: &Arc<PublishOpts>,
    receipts: &Vec<Receipt>,
) -> Vec<JoinHandle<Result<(), PublishError>>> {
    let block_height = (*opts.block_height).clone();
    let packets: Vec<PublishPacket<Log>> = receipts
        .par_iter()
        .enumerate()
        .filter_map(|(index, receipt)| match receipt {
            Receipt::Log { id, .. } | Receipt::LogData { id, .. } => {
                Some(PublishPacket::new(
                    &receipt.to_owned().into(),
                    LogsSubject {
                        block_height: Some(block_height.clone()),
                        tx_id: Some(tx_id.to_owned()),
                        receipt_index: Some(index),
                        log_id: Some((*id).into()),
                    }
                    .arc(),
                ))
            }
            _ => None,
        })
        .collect();

    packets
        .iter()
        .map(|packet| packet.publish(Arc::new(stream.to_owned()), opts))
        .collect()
}
