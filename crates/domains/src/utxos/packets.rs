use std::sync::Arc;

use async_trait::async_trait;
use fuel_core_types::fuel_types::{self, canonical::Deserialize};
use fuel_streams_types::{
    BlockTimestamp,
    ContractId,
    FuelCoreBytes32,
    FuelCoreUtxoId,
    TxId,
    UtxoId,
    UtxoStatus,
    UtxoType,
};
use rayon::prelude::*;

use super::{subjects::*, types::*, UtxosQuery};
use crate::{
    blocks::BlockHeight,
    infra::record::{PacketBuilder, RecordPacket, ToPacket},
    inputs::types::*,
    outputs::Output,
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
        let mut input_packets = tx
            .inputs
            .par_iter()
            .enumerate()
            .filter_map(|(input_index, input)| {
                let subject = DynUtxoSubject::from_input(
                    input,
                    msg_payload.block_height(),
                    tx_id.clone(),
                    *tx_index as i32,
                    input_index as i32,
                );
                let timestamp = msg_payload.timestamp();
                subject.map(|subject| {
                    let packet = subject.build_packet(timestamp);
                    match msg_payload.namespace.clone() {
                        Some(ns) => packet.with_namespace(&ns),
                        _ => packet,
                    }
                })
            })
            .collect::<Vec<RecordPacket>>();

        let output_packets = tx
            .outputs
            .par_iter()
            .enumerate()
            .filter_map(|(output_index, output)| {
                let subject = DynUtxoSubject::from_output(
                    output,
                    msg_payload.block_height(),
                    tx_id.clone(),
                    *tx_index as i32,
                    output_index as i32,
                );
                subject.map(|subject| {
                    let timestamp = msg_payload.timestamp();
                    let packet = subject.build_packet(timestamp);
                    match msg_payload.namespace.clone() {
                        Some(ns) => packet.with_namespace(&ns),
                        _ => packet,
                    }
                })
            })
            .collect::<Vec<RecordPacket>>();

        input_packets.extend(output_packets);
        input_packets
    }
}

pub struct DynUtxoSubject {
    utxo: Utxo,
    subject: UtxosSubject,
}

impl DynUtxoSubject {
    pub fn from_input(
        input: &Input,
        block_height: BlockHeight,
        tx_id: TxId,
        tx_index: i32,
        input_index: i32,
    ) -> Option<Self> {
        let item = match input {
            Input::Contract(InputContract {
                utxo_id,
                contract_id,
                ..
            }) => {
                let bytes = contract_id.clone().into_inner();
                let contract_id_wrapped =
                    ContractId::new(fuel_types::ContractId::new(*bytes));
                let utxo = Utxo {
                    status: UtxoStatus::Spent,
                    r#type: UtxoType::InputContract,
                    utxo_id: utxo_id.to_owned(),
                    tx_id: tx_id.to_owned(),
                    contract_id: Some(contract_id_wrapped.clone()),
                    from: None,
                    to: None,
                    amount: None,
                    asset_id: None,
                    nonce: None,
                };
                Some(utxo)
            }
            Input::Coin(InputCoin {
                utxo_id,
                amount,
                owner,
                asset_id,
                ..
            }) => {
                let utxo = Utxo {
                    status: UtxoStatus::Spent,
                    r#type: UtxoType::InputCoin,
                    utxo_id: utxo_id.to_owned(),
                    tx_id: tx_id.to_owned(),
                    from: Some(owner.clone()),
                    to: None,
                    amount: Some(*amount),
                    asset_id: Some(asset_id.clone()),
                    nonce: None,
                    contract_id: None,
                };
                Some(utxo)
            }
            _ => None,
        };
        match item {
            Some(utxo) => {
                let mut subject = UtxosSubject::from(utxo.clone());
                subject.block_height = Some(block_height);
                subject.tx_id = Some(tx_id);
                subject.tx_index = Some(tx_index);
                subject.input_index = Some(input_index);
                subject.output_index = Some(utxo.utxo_id.output_index as i32);
                Some(Self { utxo, subject })
            }
            None => None,
        }
    }

    pub fn from_output(
        output: &Output,
        block_height: BlockHeight,
        tx_id: TxId,
        tx_index: i32,
        output_index: i32,
    ) -> Option<Self> {
        let utxo_id = Self::build_utxo_id(&tx_id, output_index);
        let item = match output {
            Output::Coin(output) => {
                let utxo = Utxo {
                    status: UtxoStatus::Unspent,
                    r#type: UtxoType::OutputCoin,
                    tx_id: tx_id.to_owned(),
                    utxo_id: utxo_id.to_owned(),
                    from: None,
                    to: Some(output.to.to_owned()),
                    amount: Some(output.amount),
                    asset_id: Some(output.asset_id.clone()),
                    nonce: None,
                    contract_id: None,
                };
                Some(utxo)
            }
            Output::Variable(output) => {
                let utxo = Utxo {
                    status: UtxoStatus::Unspent,
                    r#type: UtxoType::OutputVariable,
                    tx_id: tx_id.to_owned(),
                    utxo_id: utxo_id.to_owned(),
                    from: None,
                    to: Some(output.to.to_owned()),
                    amount: Some(output.amount),
                    asset_id: Some(output.asset_id.clone()),
                    nonce: None,
                    contract_id: None,
                };
                Some(utxo)
            }
            Output::Change(output) => {
                let utxo = Utxo {
                    status: UtxoStatus::Unspent,
                    r#type: UtxoType::OutputChange,
                    tx_id: tx_id.to_owned(),
                    utxo_id: utxo_id.to_owned(),
                    from: None,
                    to: Some(output.to.to_owned()),
                    amount: Some(output.amount),
                    asset_id: Some(output.asset_id.clone()),
                    nonce: None,
                    contract_id: None,
                };
                Some(utxo)
            }
            _ => None,
        };
        match item {
            Some(utxo) => {
                let mut subject = UtxosSubject::from(utxo.clone());
                subject.block_height = Some(block_height);
                subject.tx_id = Some(tx_id);
                subject.tx_index = Some(tx_index);
                subject.output_index = Some(output_index);
                Some(Self { utxo, subject })
            }
            None => None,
        }
    }

    fn build_utxo_id(tx_id: &TxId, output_index: i32) -> UtxoId {
        let tx_id_bytes = tx_id.to_owned().into_inner();
        let tx_id_bytes = tx_id_bytes.as_slice();
        let tx_id_bytes = FuelCoreBytes32::from_bytes(tx_id_bytes)
            .expect("Tx ID not valid to convert for bytes");
        let utxo_id = FuelCoreUtxoId::new(tx_id_bytes, output_index as u16);
        utxo_id.into()
    }

    pub fn build_packet(
        &self,
        block_timestamp: BlockTimestamp,
    ) -> RecordPacket {
        self.utxo
            .to_packet(&Arc::new(self.subject.clone()), block_timestamp)
    }

    pub fn to_query_params(&self) -> UtxosQuery {
        UtxosQuery::from(self.subject.clone())
    }
}
