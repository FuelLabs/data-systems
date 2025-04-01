use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_subject::subject::IntoSubject;
use fuel_streams_types::{HexData, TxId};
use rayon::prelude::*;

use super::{subjects::*, Predicate};
use crate::{
    blocks::BlockHeight,
    infra::record::{PacketBuilder, RecordPacket, ToPacket},
    inputs::Input,
    transactions::Transaction,
    MsgPayload,
};

#[async_trait]
impl PacketBuilder for Predicate {
    type Opts = (MsgPayload, usize, Transaction);
    fn build_packets(
        (msg_payload, tx_index, tx): &Self::Opts,
    ) -> Vec<RecordPacket> {
        let tx_id = tx.id.clone();
        let block_height = msg_payload.block_height();
        let timestamps = msg_payload.timestamp();

        tx.inputs
            .par_iter()
            .enumerate()
            .filter_map(move |(input_index, input)| {
                let subject = DynPredicateSubject::new(
                    input,
                    &block_height,
                    &tx_id,
                    *tx_index as u32,
                    input_index as u32,
                );
                subject.map(|dyn_subject| {
                    let predicate = dyn_subject.predicate().to_owned();
                    let packet =
                        predicate.to_packet(&dyn_subject.into(), timestamps);
                    match msg_payload.namespace.clone() {
                        Some(ns) => packet.with_namespace(&ns),
                        _ => packet,
                    }
                })
            })
            .collect()
    }
}

pub struct DynPredicateSubject(Arc<dyn IntoSubject>, Predicate);
impl DynPredicateSubject {
    pub fn new(
        input: &Input,
        block_height: &BlockHeight,
        tx_id: &TxId,
        tx_index: u32,
        input_index: u32,
    ) -> Option<Self> {
        match input {
            Input::Coin(coin) => {
                let blob_id = blob_id_from_bytecode(coin.predicate.to_owned());
                let subject = PredicatesSubject {
                    block_height: Some(*block_height),
                    tx_id: Some(tx_id.to_owned()),
                    tx_index: Some(tx_index),
                    input_index: Some(input_index),
                    blob_id: blob_id.to_owned(),
                    predicate_address: Some(coin.owner.to_owned()),
                    asset: Some(coin.asset_id.to_owned()),
                };
                let predicate = Predicate::new(
                    tx_id,
                    tx_index,
                    input_index,
                    blob_id,
                    &coin.owner,
                    &coin.predicate,
                    &coin.asset_id,
                );
                Some(Self(subject.arc(), predicate))
            }
            _ => None,
        }
    }

    pub fn predicate(&self) -> &Predicate {
        &self.1
    }
}
impl From<DynPredicateSubject> for Arc<dyn IntoSubject> {
    fn from(subject: DynPredicateSubject) -> Self {
        subject.0
    }
}

pub(crate) fn blob_id_from_bytecode(bytecode: HexData) -> Option<HexData> {
    let bytes = bytecode.into_inner();
    let bytes = bytes.as_ref();
    let value = super::utils::extract_blob_id_and_section_offset(bytes)
        .expect("Failed to parse predicate bytecode");
    value.map(|(b, _)| HexData::from(b.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::blob_id_from_bytecode;

    const VALID_BYTECODE: &str = "1a403000504100301a445000ba49000032400481504100205d490000504100083240048220451300524510044a440000cf534ed3e0f17779f12866863001e53beb68e87621308fbe7f575652dba0dc000000000000000108f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad0700000000000000010000000000002710666c984d4c0aa70abb14a6d6e7693fc5bda8275d6c6716c8bcae33b3c21a1dfb6fd333a74ac52ca7d50d7e768996acd0fb339fcc8a109796b2c55d2f341631d3a0265fb5c32f6e8db3197af3c7eb05c48ae373605b8165b6f4a51c5b0ba4812edfda4cd39004d68b93c8be7a82f67c18661345e0b8e03a479a9eb4118277c2f190d67a87f1def93ab95e5d940d1534e2d9fed411ba78c9add53930d5b567d3b2cccccccccccc00020000000000000000000000000000000000000000000000000000000000000000000000000000158c0000000000000cf4";
    const INVALID_BYTECODE: &str = "1a40500091000020614400096148000342411480504cc04c72580020295134165b501012615c000572680002595d7001616172005b61a010616572455b6400125b5c100b24040000240000007cc480c6385fe2c31dc95cc830e4ffb75da5532558ef938b8368da930bf60722";

    #[test]
    fn test_extract_blob_id() -> anyhow::Result<()> {
        let bytes = hex::decode(VALID_BYTECODE)?;
        let blob_id = blob_id_from_bytecode(bytes.into());
        assert!(blob_id.is_some());

        let bytes = hex::decode(INVALID_BYTECODE)?;
        let blob_id = blob_id_from_bytecode(bytes.into());
        assert!(blob_id.is_none());
        Ok(())
    }
}
