use std::cmp::Ordering;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::{BlockHeight, BlockTimestamp};
use serde::{Deserialize, Serialize};

use super::subjects::*;
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
    pub tx_id: String,
    pub tx_index: i32,
    pub input_index: i32,
    pub utxo_type: String,
    pub utxo_id: String,
    pub contract_id: Option<String>,
    pub created_at: BlockTimestamp,
    pub published_at: BlockTimestamp,
}

impl DataEncoder for UtxoDbItem {}

impl DbItem for UtxoDbItem {
    fn cursor(&self) -> Cursor {
        Cursor::new(&[&self.block_height, &self.tx_index, &self.input_index])
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

    fn published_at(&self) -> BlockTimestamp {
        self.published_at
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

        match subject {
            Subjects::Utxos(subject) => Ok(UtxoDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i32,
                input_index: subject.input_index.unwrap() as i32,
                utxo_type: subject.utxo_type.unwrap().to_string(),
                utxo_id: subject.utxo_id.unwrap().to_string(),
                contract_id: subject.contract_id.map(|id| id.to_string()),
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
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
        // Order by block height first
        self.block_height
            .cmp(&other.block_height)
            // Then by transaction index within the block
            .then(self.tx_index.cmp(&other.tx_index))
            // Finally by input index within the transaction
            .then(self.input_index.cmp(&other.input_index))
    }
}

impl From<UtxoDbItem> for RecordPointer {
    fn from(val: UtxoDbItem) -> Self {
        RecordPointer {
            block_height: val.block_height,
            tx_index: Some(val.tx_index as u32),
            input_index: Some(val.input_index as u32),
            output_index: None,
            receipt_index: None,
        }
    }
}
