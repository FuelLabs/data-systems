use std::sync::Arc;

use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::{
    packets::{PublishError, PublishOpts, PublishPacket},
    FuelCoreLike,
};

pub fn publish_tasks(
    tx: &Transaction,
    stream: &Stream<Log>,
    opts: &Arc<PublishOpts>,
    fuel_core: &dyn FuelCoreLike,
) -> Vec<JoinHandle<Result<(), PublishError>>> {
    let tx_id = tx.id(&opts.chain_id);
    let block_height = (*opts.block_height).clone();
    let receipts = fuel_core.get_receipts(&tx_id).unwrap_or_default();
    let packets: Vec<PublishPacket<Log>> = receipts
        .unwrap_or_default()
        .par_iter()
        .enumerate()
        .filter_map(|(index, receipt)| match receipt {
            Receipt::Log { id, .. } | Receipt::LogData { id, .. } => {
                Some(PublishPacket::new(
                    &receipt.to_owned().into(),
                    LogsSubject::new()
                        .with_block_height(Some(block_height.clone()))
                        .with_tx_id(Some(tx_id.into()))
                        .with_receipt_index(Some(index))
                        .with_log_id(Some((*id).into()))
                        .arc(),
                    LogsSubject::WILDCARD,
                ))
            }
            _ => None,
        })
        .collect();

    packets
        .iter()
        .map(|packet| {
            packet.publish(Arc::new(stream.to_owned()), Arc::clone(opts))
        })
        .collect()
}
