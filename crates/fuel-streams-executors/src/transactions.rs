use fuel_streams_core::{subjects::*, types::*};
use fuel_streams_store::store::StorePacket;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Transaction> {
    pub fn process(
        &self,
        (tx_index, tx): (usize, &Transaction),
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let block_height = self.block_height();
        let order = self.record_order().with_tx(tx_index as u32);
        packets_from_tx((tx_index, tx), &block_height, &order)
            .iter()
            .map(|packet| self.publish(packet))
            .collect()
    }
}

fn packets_from_tx(
    (tx_index, tx): (usize, &Transaction),
    block_height: &BlockHeight,
    tx_order: &RecordOrder,
) -> Vec<StorePacket<Transaction>> {
    let estimated_capacity =
        1 + tx.inputs.len() + tx.outputs.len() + tx.receipts.len();

    let tx_id = tx.id.clone();
    let tx_status = tx.status.clone();
    let receipts = tx.receipts.clone();

    // Main subject
    let index_u8 = tx_index as u8;
    let mut packets = Vec::with_capacity(estimated_capacity);
    let main_subject = TransactionsSubject {
        block_height: Some(block_height.to_owned()),
        index: Some(tx_index),
        tx_id: Some(tx_id.to_owned()),
        status: Some(tx_status),
        kind: Some(tx.kind.to_owned()),
    }
    .parse();
    packets.push(tx.to_packet(main_subject, tx_order));

    let tx_ids = rayon::iter::once(&tx.kind)
        .map(|kind| {
            let ids = identifiers(tx, kind, &tx_id, index_u8);
            (ids, tx_order.clone())
        })
        .collect::<Vec<_>>();

    let input_ids = tx
        .inputs
        .par_iter()
        .enumerate()
        .map(|(index, input)| {
            let ids = inputs::identifiers(input, &tx_id, index_u8);
            let order = tx_order.clone().with_record(index as u32);
            (ids, order)
        })
        .collect::<Vec<_>>();

    let output_ids = tx
        .outputs
        .par_iter()
        .enumerate()
        .map(|(index, output)| {
            let ids = outputs::identifiers(output, tx, &tx_id, index_u8);
            let order = tx_order.clone().with_record(index as u32);
            (ids, order)
        })
        .collect::<Vec<_>>();

    let receipt_ids = receipts
        .par_iter()
        .enumerate()
        .map(|(index, receipt)| {
            let ids = receipts::identifiers(receipt, &tx_id, index_u8);
            let order = tx_order.clone().with_record(index as u32);
            (ids, order)
        })
        .collect::<Vec<_>>();

    let mut additional_packets = tx_ids
        .into_iter()
        .chain(input_ids)
        .chain(output_ids)
        .chain(receipt_ids)
        .flat_map(|(ids, order)| {
            ids.into_iter()
                .map(|id| id.into())
                .map(|subject: TransactionsByIdSubject| {
                    tx.to_packet(subject.parse(), &order)
                })
                .collect::<Vec<_>>()
        })
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
