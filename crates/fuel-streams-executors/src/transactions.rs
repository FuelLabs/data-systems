use fuel_streams_core::prelude::*;
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
) -> Vec<PublishPacket<Transaction>> {
    let tx_id = tx.id.clone();
    let tx_status = tx.status.clone();
    let receipts = tx.receipts.clone();
    let main_subject = TransactionsSubject {
        block_height: Some(block_height.to_owned()),
        index: Some(index),
        tx_id: Some(tx_id.to_owned()),
        status: Some(tx_status.to_owned()),
        kind: Some(tx.kind.to_owned()),
    }
    .arc();

    let mut packets = vec![tx.to_packet(main_subject)];
    packets.extend(
        identifiers(tx, &tx.kind, &tx_id, index as u8)
            .into_par_iter()
            .map(|identifier| identifier.into())
            .map(|subject: TransactionsByIdSubject| subject.arc())
            .map(|subject| tx.to_packet(subject))
            .collect::<Vec<_>>(),
    );

    let packets_from_inputs: Vec<PublishPacket<Transaction>> = tx
        .inputs
        .par_iter()
        .flat_map(|input| {
            inputs::identifiers(input, &tx_id, index as u8)
                .into_par_iter()
                .map(|identifier| identifier.into())
                .map(|subject: TransactionsByIdSubject| subject.arc())
                .map(|subject| tx.to_packet(subject))
        })
        .collect();
    packets.extend(packets_from_inputs);

    let packets_from_outputs: Vec<PublishPacket<Transaction>> = tx
        .outputs
        .par_iter()
        .flat_map(|output| {
            outputs::identifiers(output, tx, &tx_id, index as u8)
                .into_par_iter()
                .map(|identifier| identifier.into())
                .map(|subject: TransactionsByIdSubject| subject.arc())
                .map(|subject| tx.to_packet(subject))
        })
        .collect();
    packets.extend(packets_from_outputs);

    let packets_from_receipts: Vec<PublishPacket<Transaction>> = receipts
        .par_iter()
        .flat_map(|receipt| {
            receipts::identifiers(receipt, &tx_id, index as u8)
                .into_par_iter()
                .map(|identifier| identifier.into())
                .map(|subject: TransactionsByIdSubject| subject.arc())
                .map(|subject| tx.to_packet(subject))
        })
        .collect();
    packets.extend(packets_from_receipts);
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
            let script_tag = sha256(script_data);
            vec![Identifier::ScriptID(tx_id.to_owned(), index, script_tag)]
        }
        _ => Vec::new(),
    }
}
