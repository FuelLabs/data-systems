use std::cmp::Ordering;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::{subjects::*, Utxo};
use crate::{
    infra::{
        db::DbItem,
        record::{
            RecordEntity,
            RecordPacket,
            RecordPacketError,
            RecordPointer,
        },
        Cursor,
        DbError,
    },
    Subjects,
};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct UtxoDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: BlockHeight,
    pub tx_id: TxId,
    pub tx_index: i32,
    pub output_index: i32,

    pub utxo_id: String,
    pub r#type: UtxoType,
    pub status: UtxoStatus,
    pub amount: Option<i64>,
    pub asset_id: Option<String>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub contract_id: Option<String>,
    pub nonce: Option<String>,

    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
}

impl DataEncoder for UtxoDbItem {}

impl DbItem for UtxoDbItem {
    fn cursor(&self) -> Cursor {
        Cursor::new(&[&self.block_height, &self.tx_index, &self.output_index])
    }

    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Utxo
    }

    fn encoded_value(&self) -> Result<Vec<u8>, DbError> {
        Ok(self.value.clone())
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }

    fn subject_id(&self) -> String {
        UtxosSubject::ID.to_string()
    }

    fn created_at(&self) -> BlockTimestamp {
        self.created_at
    }

    fn block_time(&self) -> BlockTimestamp {
        self.block_time
    }

    fn block_height(&self) -> BlockHeight {
        self.block_height
    }
}

impl TryFrom<&RecordPacket> for UtxoDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        if let Subjects::Utxos(subject) = subject {
            let utxo = Utxo::decode_json(&packet.value)?;
            Ok(UtxoDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: packet.pointer.block_height,
                tx_id: packet.pointer.tx_id.to_owned().unwrap(),
                tx_index: packet.pointer.tx_index.unwrap() as i32,
                output_index: subject.output_index.unwrap(),

                utxo_id: utxo.utxo_id.to_string(),
                r#type: utxo.r#type,
                status: utxo.status,
                amount: utxo.amount.map(|a| a.into_inner() as i64),
                asset_id: utxo.asset_id.map(|a| a.to_string()),
                from_address: utxo.from.map(|f| f.to_string()),
                to_address: utxo.to.map(|t| t.to_string()),
                contract_id: utxo.contract_id.map(|c| c.to_string()),
                nonce: utxo.nonce.map(|n| n.to_string()),

                block_time: packet.block_timestamp,
                created_at: packet.block_timestamp,
            })
        } else {
            Err(RecordPacketError::SubjectMismatch)
        }
    }
}

impl PartialOrd for UtxoDbItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UtxoDbItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.block_height
            .cmp(&other.block_height)
            .then(self.tx_index.cmp(&other.tx_index))
            .then(self.output_index.cmp(&other.output_index))
    }
}

impl From<UtxoDbItem> for RecordPointer {
    fn from(val: UtxoDbItem) -> Self {
        RecordPointer {
            block_height: val.block_height,
            tx_id: Some(val.tx_id),
            tx_index: Some(val.tx_index as u32),
            input_index: None,
            output_index: Some(val.output_index as u32),
            receipt_index: None,
        }
    }
}
