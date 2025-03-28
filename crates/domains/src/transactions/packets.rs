use async_trait::async_trait;
use fuel_streams_store::record::{PacketBuilder, Record, RecordPacket};
use rayon::prelude::*;

use super::{subjects::*, Transaction};
use crate::{
    inputs::Input,
    outputs::Output,
    predicates::Predicate,
    receipts::Receipt,
    utxos::Utxo,
    MsgPayload,
};

#[async_trait]
impl PacketBuilder for Transaction {
    type Opts = MsgPayload;

    fn build_packets(msg_payload: &Self::Opts) -> Vec<RecordPacket> {
        msg_payload
            .transactions
            .par_iter()
            .enumerate()
            .flat_map_iter(|(tx_index, tx)| {
                let sub_items_params =
                    (msg_payload.clone(), tx_index, tx.to_owned());
                let tx_packet = main_tx_packet(msg_payload, tx, tx_index);
                let input_packets = Input::build_packets(&sub_items_params);
                let output_packets = Output::build_packets(&sub_items_params);
                let receipt_packets = Receipt::build_packets(&sub_items_params);
                let utxos_packets = Utxo::build_packets(&sub_items_params);
                let predicate_packets =
                    Predicate::build_packets(&sub_items_params);
                tx_packet
                    .into_iter()
                    .chain(input_packets)
                    .chain(output_packets)
                    .chain(receipt_packets)
                    .chain(utxos_packets)
                    .chain(predicate_packets)
            })
            .collect()
    }
}

pub fn main_tx_packet(
    msg_payload: &MsgPayload,
    tx: &Transaction,
    tx_index: usize,
) -> Vec<RecordPacket> {
    let tx_id = tx.id.clone();
    let tx_status = tx.status.clone();
    let subject = TransactionsSubject {
        block_height: Some(msg_payload.block_height()),
        tx_index: Some(tx_index as u32),
        tx_id: Some(tx_id.to_owned()),
        tx_status: Some(tx_status),
        tx_type: Some(tx.tx_type.to_owned()),
    }
    .dyn_arc();
    let timestamps = msg_payload.timestamp();
    let packet = tx.to_packet(&subject, timestamps);
    let packet = match msg_payload.namespace.clone() {
        Some(ns) => packet.with_namespace(&ns),
        _ => packet,
    };
    std::iter::once(packet).collect()
}
