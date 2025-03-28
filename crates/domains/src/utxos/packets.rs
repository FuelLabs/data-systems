use std::sync::Arc;

use async_trait::async_trait;
use fuel_core_types::fuel_types;
use fuel_streams_store::record::{PacketBuilder, Record, RecordPacket};
use fuel_streams_subject::subject::IntoSubject;
use fuel_streams_types::{ContractId, TxId};
use rayon::prelude::*;

use super::{subjects::*, types::*};
use crate::{
    blocks::BlockHeight,
    inputs::types::*,
    transactions::Transaction,
    MsgPayload,
};

#[async_trait]
impl PacketBuilder for Utxo {
    type Opts = (MsgPayload, usize, Transaction);
    fn build_packets(
        (msg_payload, tx_index, tx): &Self::Opts,
    ) -> Vec<RecordPacket> {
        let tx_id = tx.id.clone();
        tx.inputs
            .par_iter()
            .enumerate()
            .map(|(input_index, input)| {
                let subject = DynUtxoSubject::from((
                    input,
                    msg_payload.block_height(),
                    tx_id.clone(),
                    *tx_index as u32,
                    input_index as u32,
                ));
                let timestamp = msg_payload.timestamp();
                let packet =
                    subject.utxo().to_packet(subject.subject(), timestamp);
                match msg_payload.namespace.clone() {
                    Some(ns) => packet.with_namespace(&ns),
                    _ => packet,
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct DynUtxoSubject(Utxo, Arc<dyn IntoSubject>);
impl From<(&Input, BlockHeight, TxId, u32, u32)> for DynUtxoSubject {
    fn from(
        (input, block_height, tx_id, tx_index, input_index): (
            &Input,
            BlockHeight,
            TxId,
            u32,
            u32,
        ),
    ) -> Self {
        match input {
            Input::Contract(InputContract {
                utxo_id,
                contract_id,
                ..
            }) => {
                let bytes = contract_id.clone().into_inner();
                let contract_id_wrapped =
                    ContractId::new(fuel_types::ContractId::new(*bytes));
                let utxo = Utxo {
                    utxo_id: utxo_id.to_owned(),
                    tx_id: tx_id.to_owned(),
                    contract_id: Some(contract_id_wrapped.clone()),
                    ..Default::default()
                };
                let subject = UtxosSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    input_index: Some(input_index),
                    utxo_type: Some(UtxoType::Contract),
                    utxo_id: Some(utxo_id.into()),
                    contract_id: Some(contract_id_wrapped),
                }
                .arc();
                DynUtxoSubject(utxo, subject)
            }
            Input::Coin(InputCoin {
                utxo_id, amount, ..
            }) => {
                let utxo = Utxo {
                    utxo_id: utxo_id.to_owned(),
                    amount: Some(*amount),
                    tx_id: tx_id.to_owned(),
                    ..Default::default()
                };
                let subject = UtxosSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    input_index: Some(input_index),
                    utxo_type: Some(UtxoType::Coin),
                    utxo_id: Some(utxo_id.into()),
                    ..Default::default()
                }
                .arc();
                DynUtxoSubject(utxo, subject)
            }
            Input::Message(
                input @ InputMessage {
                    amount,
                    nonce,
                    recipient,
                    sender,
                    data,
                    ..
                },
            ) => {
                let utxo_id = input.computed_utxo_id();
                let utxo = Utxo {
                    tx_id: tx_id.to_owned(),
                    utxo_id: utxo_id.to_owned(),
                    sender: Some(sender.to_owned()),
                    recipient: Some(recipient.to_owned()),
                    nonce: Some(nonce.to_owned()),
                    amount: Some(*amount),
                    data: Some(data.to_owned()),
                    ..Default::default()
                };
                let subject = UtxosSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    input_index: Some(input_index),
                    utxo_type: Some(UtxoType::Message),
                    utxo_id: Some(utxo_id.into()),
                    ..Default::default()
                }
                .arc();
                DynUtxoSubject(utxo, subject)
            }
        }
    }
}

impl DynUtxoSubject {
    pub fn utxo(&self) -> &Utxo {
        &self.0
    }

    pub fn subject(&self) -> &Arc<dyn IntoSubject> {
        &self.1
    }
}
