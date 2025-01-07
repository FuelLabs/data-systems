use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Log> {
    pub fn process(
        &self,
        (tx_index, tx): (usize, &Transaction),
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let block_height = self.block_height();
        let tx_id = tx.id.clone();
        let receipts = tx.receipts.clone();
        let packets = receipts
            .par_iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                Receipt::Log(LogReceipt { id, .. })
                | Receipt::LogData(LogDataReceipt { id, .. }) => {
                    let order = self
                        .record_order()
                        .with_tx(tx_index as u32)
                        .with_record(index as u32);

                    Some(Log::to_packet(
                        &receipt.into(),
                        LogsSubject {
                            block_height: Some(block_height.clone()),
                            tx_id: Some(tx_id.to_owned()),
                            receipt_index: Some(index),
                            log_id: Some(id.into()),
                        }
                        .parse(),
                        &order,
                    ))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        packets.iter().map(|packet| self.publish(packet)).collect()
    }
}
