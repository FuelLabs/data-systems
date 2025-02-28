use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_store::record::{PacketBuilder, Record, RecordPacket};
use fuel_streams_subject::subject::IntoSubject;
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
        let tx_id = tx.id.clone();
        tx.inputs
            .par_iter()
            .enumerate()
            .map(move |(input_index, input)| {
                let subject = DynInputSubject::from((
                    input,
                    msg_payload.block_height(),
                    tx_id.clone(),
                    *tx_index as u32,
                    input_index as u32,
                ));

                let timestamps = msg_payload.timestamp();
                let packet = input.to_packet(&subject.into(), timestamps);
                match msg_payload.namespace.clone() {
                    Some(ns) => packet.with_namespace(&ns),
                    _ => packet,
                }
            })
            .collect()
    }
}

pub struct DynInputSubject(Arc<dyn IntoSubject>);
impl From<(&Input, BlockHeight, TxId, u32, u32)> for DynInputSubject {
    fn from(
        (input, block_height, tx_id, tx_index, input_index): (
            &Input,
            BlockHeight,
            TxId,
            u32,
            u32,
        ),
    ) -> Self {
        DynInputSubject(match input {
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
        })
    }
}

impl From<DynInputSubject> for Arc<dyn IntoSubject> {
    fn from(subject: DynInputSubject) -> Self {
        subject.0
    }
}
