use std::cmp::Ordering;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::{BlockHeight, BlockTimestamp};
use serde::{Deserialize, Serialize};

use super::{subjects::*, Predicate};
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
pub struct PredicateDbItem {
    pub subject: String,
    pub block_height: BlockHeight,
    pub tx_id: String,
    pub tx_index: i32,
    pub input_index: i32,
    // predicate types properties
    pub blob_id: Option<String>,
    pub predicate_address: String,
    pub asset_id: String,
    pub bytecode: String,
    pub created_at: BlockTimestamp,
    pub published_at: BlockTimestamp,
}

impl DataEncoder for PredicateDbItem {}

impl DbItem for PredicateDbItem {
    fn cursor(&self) -> Cursor {
        Cursor::new(&[&self.block_height, &self.tx_index, &self.input_index])
    }

    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Predicate
    }

    fn encoded_value(&self) -> Result<Vec<u8>, DbError> {
        Ok(Predicate::try_from(self)?.encode_json()?)
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }

    fn subject_id(&self) -> String {
        PredicatesSubject::ID.to_string()
    }

    fn created_at(&self) -> BlockTimestamp {
        self.created_at
    }

    fn block_time(&self) -> BlockTimestamp {
        self.published_at
    }

    fn block_height(&self) -> BlockHeight {
        self.block_height
    }
}

impl TryFrom<&RecordPacket> for PredicateDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        let predicate =
            Predicate::decode_json(&packet.value).map_err(|_| {
                RecordPacketError::DecodeFailed(packet.subject_str())
            })?;

        match subject {
            Subjects::Predicates(subject) => Ok(PredicateDbItem {
                subject: packet.subject_str(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                input_index: subject.input_index.unwrap(),
                blob_id: subject.blob_id.map(|b| b.to_string()),
                predicate_address: subject
                    .predicate_address
                    .unwrap()
                    .to_string(),
                bytecode: predicate.predicate_bytecode.to_string(),
                asset_id: subject.asset.unwrap_or_default().to_string(),
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
        }
    }
}

impl PartialOrd for PredicateDbItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PredicateDbItem {
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

impl From<PredicateDbItem> for RecordPointer {
    fn from(val: PredicateDbItem) -> Self {
        RecordPointer {
            block_height: val.block_height,
            tx_index: Some(val.tx_index as u32),
            input_index: Some(val.input_index as u32),
            output_index: None,
            receipt_index: None,
        }
    }
}
