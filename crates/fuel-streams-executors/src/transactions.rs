use fuel_streams_core::{subjects::*, types::*};
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Transaction> {
    pub fn process(
        &self,
        (tx_index, tx): (usize, &Transaction),
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let block_height = self.block_height();
        let packet = packet_from_tx((tx_index, tx), &block_height);
        vec![self.publish(&packet)]
    }
}

fn packet_from_tx(
    (tx_index, tx): (usize, &Transaction),
    block_height: &BlockHeight,
) -> RecordPacket<Transaction> {
    let tx_id = tx.id.clone();
    let tx_status = tx.status.clone();
    let main_subject = TransactionsSubject {
        block_height: Some(block_height.to_owned()),
        tx_index: Some(tx_index as u32),
        tx_id: Some(tx_id.to_owned()),
        status: Some(tx_status),
        kind: Some(tx.kind.to_owned()),
    }
    .arc();
    tx.to_packet(main_subject)
}

// fn identifiers(
//     tx: &transaction,
//     kind: &transactionkind,
//     tx_id: &bytes32,
//     index: u8,
// ) -> vec<identifier> {
//     match kind {
//         transactionkind::script => {
//             let script_data = &tx.script_data.to_owned().unwrap_or_default().0;
//             let script_tag = sha256(&script_data.0);
//             vec![identifier::scriptid(tx_id.to_owned(), index, script_tag)]
//         }
//         _ => vec::new(),
//     }
// }
