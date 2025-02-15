use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, DbItem},
    record::{
        DataEncoder,
        RecordEntity,
        RecordPacket,
        RecordPacketError,
        RecordPointer,
    },
};
use serde::{Deserialize, Serialize};

use super::subjects::*;
use crate::Subjects;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct TransactionDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_id: String,
    pub tx_index: i32,
    pub tx_status: String,
    pub kind: String,
}

impl DataEncoder for TransactionDbItem {
    type Err = DbError;
}

impl DbItem for TransactionDbItem {
    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Transaction
    }

    fn encoded_value(&self) -> &[u8] {
        &self.value
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }

    fn subject_id(&self) -> String {
        TransactionsSubject::ID.to_string()
    }
}

impl TryFrom<&RecordPacket> for TransactionDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::Transactions(subject) => Ok(TransactionDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i32,
                tx_status: subject.tx_status.unwrap().to_string(),
                kind: subject.kind.unwrap().to_string(),
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
        }
    }
}

impl PartialOrd for TransactionDbItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TransactionDbItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order by block height first
        self.block_height
            .cmp(&other.block_height)
            // Then by transaction index within the block
            .then(self.tx_index.cmp(&other.tx_index))
    }
}

impl From<TransactionDbItem> for RecordPointer {
    fn from(val: TransactionDbItem) -> Self {
        RecordPointer {
            block_height: val.block_height.into(),
            tx_index: Some(val.tx_index as u32),
            input_index: None,
            output_index: None,
            receipt_index: None,
        }
    }
}
