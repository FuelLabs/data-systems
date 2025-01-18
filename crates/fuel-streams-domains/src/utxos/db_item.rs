use fuel_streams_store::{
    db::{DbError, DbItem},
    record::{DataEncoder, RecordEntity, RecordPacket, RecordPacketError},
};
use serde::{Deserialize, Serialize};

use crate::Subjects;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct UtxoDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_id: String,
    pub tx_index: i64,
    pub input_index: i64,
    pub utxo_type: String,
    pub utxo_id: String,
}

impl DataEncoder for UtxoDbItem {
    type Err = DbError;
}

impl DbItem for UtxoDbItem {
    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Utxo
    }

    fn encoded_value(&self) -> &[u8] {
        &self.value
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }
}

impl TryFrom<&RecordPacket> for UtxoDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::Utxos(subject) => Ok(UtxoDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                input_index: subject.input_index.unwrap() as i64,
                utxo_type: subject.utxo_type.unwrap().to_string(),
                utxo_id: subject.utxo_id.unwrap().to_string(),
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
        }
    }
}
