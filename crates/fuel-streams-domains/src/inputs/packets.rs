use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_macros::subject::IntoSubject;
use fuel_streams_store::record::{PacketBuilder, Record, RecordPacket};
use fuel_streams_types::TxId;
use rayon::prelude::*;

use super::{subjects::*, Input};
use crate::{blocks::BlockHeight, transactions::Transaction, MsgPayload};

#[async_trait]
impl PacketBuilder for Input {
    type Opts = (MsgPayload, usize, Transaction);
    fn build_packets(
        (msg_payload, tx_index, tx): &Self::Opts,
    ) -> Vec<RecordPacket> {
        let block_height = msg_payload.block_height();
        let tx_id = tx.id.clone();
        tx.inputs
            .par_iter()
            .enumerate()
            .map(move |(input_index, input)| {
                let subject = main_subject(
                    block_height.clone(),
                    *tx_index as u32,
                    input_index as u32,
                    tx_id.clone(),
                    input,
                );
                let packet = input.to_packet(&subject);
                match msg_payload.namespace.clone() {
                    Some(ns) => packet.with_namespace(&ns),
                    _ => packet,
                }
            })
            .collect()
    }
}

fn main_subject(
    block_height: BlockHeight,
    tx_index: u32,
    input_index: u32,
    tx_id: TxId,
    input: &Input,
) -> Arc<dyn IntoSubject> {
    match input {
        Input::Contract(contract) => InputsContractSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            input_index: Some(input_index),
            contract: Some(contract.contract_id.to_owned().into()),
        }
        .arc(),
        Input::Coin(coin) => InputsCoinSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            input_index: Some(input_index),
            owner: Some(coin.owner.to_owned()),
            asset: Some(coin.asset_id.to_owned()),
        }
        .arc(),
        Input::Message(message) => InputsMessageSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            input_index: Some(input_index),
            sender: Some(message.sender.to_owned()),
            recipient: Some(message.recipient.to_owned()),
        }
        .arc(),
    }
}
