use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::prelude::*;

impl Executor<Log> {
    pub fn process(
        &self,
        tx: &Transaction,
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
                    Some(PublishPacket::new(
                        receipt.to_owned().into(),
                        LogsSubject {
                            block_height: Some(block_height.clone()),
                            tx_id: Some(tx_id.to_owned()),
                            receipt_index: Some(index),
                            log_id: Some(id.into()),
                        }
                        .arc(),
                    ))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        packets.iter().map(|packet| self.publish(packet)).collect()
    }
}
