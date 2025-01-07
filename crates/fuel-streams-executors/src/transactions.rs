use fuel_streams_core::prelude::*;
use fuel_streams_store::store::StorePacket;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Transaction> {
    pub fn process(
        &self,
        tx_item: (usize, &Transaction),
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let block_height = self.block_height();
        packets_from_tx(tx_item, &block_height)
            .iter()
            .map(|packet| self.publish(packet))
            .collect()
    }
}

fn packets_from_tx(
    (index, tx): (usize, &Transaction),
    block_height: &BlockHeight,
) -> Vec<StorePacket<Transaction>> {
    let estimated_capacity =
        1 + tx.inputs.len() + tx.outputs.len() + tx.receipts.len();

    let tx_id = tx.id.clone();
    let tx_status = tx.status.clone();
    let receipts = tx.receipts.clone();

    // Main subject
    let mut packets = Vec::with_capacity(estimated_capacity);
    packets.push(
        tx.to_packet(
            TransactionsSubject {
                block_height: Some(block_height.to_owned()),
                index: Some(index),
                tx_id: Some(tx_id.to_owned()),
                status: Some(tx_status),
                kind: Some(tx.kind.to_owned()),
            }
            .parse(),
        ),
    );

    let index_u8 = index as u8;
    let mut additional_packets = rayon::iter::once(&tx.kind)
        .flat_map(|kind| identifiers(tx, kind, &tx_id, index_u8))
        .chain(
            tx.inputs
                .par_iter()
                .flat_map(|input| inputs::identifiers(input, &tx_id, index_u8)),
        )
        .chain(tx.outputs.par_iter().flat_map(|output| {
            outputs::identifiers(output, tx, &tx_id, index_u8)
        }))
        .chain(receipts.par_iter().flat_map(|receipt| {
            receipts::identifiers(receipt, &tx_id, index_u8)
        }))
        .map(|identifier| identifier.into())
        .map(|subject: TransactionsByIdSubject| tx.to_packet(subject.parse()))
        .collect::<Vec<_>>();

    packets.append(&mut additional_packets);
    packets
}

fn identifiers(
    tx: &Transaction,
    kind: &TransactionKind,
    tx_id: &Bytes32,
    index: u8,
) -> Vec<Identifier> {
    match kind {
        TransactionKind::Script => {
            let script_data = &tx.script_data.to_owned().unwrap_or_default().0;
            let script_tag = sha256(&script_data.0);
            vec![Identifier::ScriptID(tx_id.to_owned(), index, script_tag)]
        }
        _ => Vec::new(),
    }
}
