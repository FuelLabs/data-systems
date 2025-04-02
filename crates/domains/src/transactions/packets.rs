use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_types::{BlockHeight, BlockTimestamp};
use rayon::prelude::*;

use super::{subjects::*, Transaction, TransactionsQuery};
use crate::{
    infra::record::{PacketBuilder, RecordPacket, ToPacket},
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
                let tx_packet =
                    main_tx_packet(msg_payload, tx, tx_index as i32);
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

pub enum DynTransactionSubject {
    Transaction(TransactionsSubject),
}

impl DynTransactionSubject {
    pub fn new(
        tx: &Transaction,
        block_height: BlockHeight,
        tx_index: i32,
    ) -> Self {
        let tx_id = tx.id.clone();
        let tx_status = tx.status.clone();
        Self::Transaction(TransactionsSubject {
            block_height: Some(block_height),
            tx_index: Some(tx_index),
            tx_id: Some(tx_id),
            tx_status: Some(tx_status),
            tx_type: Some(tx.r#type.to_owned()),
        })
    }

    pub fn build_packet(
        &self,
        tx: &Transaction,
        block_timestamp: BlockTimestamp,
    ) -> RecordPacket {
        match self {
            Self::Transaction(subject) => {
                tx.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
        }
    }

    pub fn to_query_params(&self) -> TransactionsQuery {
        match self {
            Self::Transaction(subject) => {
                TransactionsQuery::from(subject.to_owned())
            }
        }
    }
}

pub fn main_tx_packet(
    msg_payload: &MsgPayload,
    tx: &Transaction,
    tx_index: i32,
) -> Vec<RecordPacket> {
    let block_height = msg_payload.block_height();
    let subject = DynTransactionSubject::new(tx, block_height, tx_index);
    let timestamps = msg_payload.timestamp();
    let packet = subject.build_packet(tx, timestamps);
    let packet = match msg_payload.namespace.clone() {
        Some(ns) => packet.with_namespace(&ns),
        _ => packet,
    };
    std::iter::once(packet).collect()
}
