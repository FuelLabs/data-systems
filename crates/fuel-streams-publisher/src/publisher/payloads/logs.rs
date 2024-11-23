use std::sync::Arc;

use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::{publish, PublishOpts};

pub fn publish_tasks(
    tx_id: &Bytes32,
    stream: &Stream<Log>,
    opts: &Arc<PublishOpts>,
    receipts: &Vec<FuelCoreReceipt>,
) -> Vec<JoinHandle<anyhow::Result<()>>> {
    let block_height = (*opts.block_height).clone();
    let packets = receipts
        .par_iter()
        .enumerate()
        .filter_map(|(index, receipt)| match receipt {
            FuelCoreReceipt::Log { id, .. }
            | FuelCoreReceipt::LogData { id, .. } => Some(PublishPacket::new(
                receipt.to_owned().into(),
                LogsSubject {
                    block_height: Some(block_height.clone()),
                    tx_id: Some(tx_id.to_owned()),
                    receipt_index: Some(index),
                    log_id: Some((*id).into()),
                }
                .arc(),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();

    packets
        .iter()
        .map(|packet| publish(packet, Arc::new(stream.to_owned()), opts))
        .collect()
}
