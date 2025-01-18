use fuel_streams_store::{
    db::{DbError, DbItem},
    record::{DataEncoder, RecordEntity, RecordPacket, RecordPacketError},
};
use serde::{Deserialize, Serialize};

use crate::Subjects;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct TransactionDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_id: String,
    pub tx_index: i64,
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
}

impl TryFrom<&RecordPacket> for TransactionDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::Transactions(subject) => Ok(TransactionDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                tx_status: subject.tx_status.unwrap().to_string(),
                kind: subject.kind.unwrap().to_string(),
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
        }
    }
}
