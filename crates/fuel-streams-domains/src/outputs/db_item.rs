use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, DbItem},
    record::{DataEncoder, RecordEntity, RecordPacket, RecordPacketError},
};
use serde::{Deserialize, Serialize};

use super::Output;
use crate::Subjects;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct OutputDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_id: String,
    pub tx_index: i64,
    pub output_index: i64,
    pub output_type: String,
    pub to_address: Option<String>, // for coin, change, and variable outputs
    pub asset_id: Option<String>,   // for coin, change, and variable outputs
    pub contract_id: Option<String>, /* for contract and contract_created outputs */
}

impl DataEncoder for OutputDbItem {
    type Err = DbError;
}

impl DbItem for OutputDbItem {
    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Output
    }

    fn encoded_value(&self) -> &[u8] {
        &self.value
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }
}

impl PartialOrd for OutputDbItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OutputDbItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order by block height first
        self.block_height
            .cmp(&other.block_height)
            // Then by transaction index within the block
            .then(self.tx_index.cmp(&other.tx_index))
            // Finally by output index within the transaction
            .then(self.output_index.cmp(&other.output_index))
    }
}

impl TryFrom<&RecordPacket<Output>> for OutputDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket<Output>) -> Result<Self, Self::Error> {
        let record = packet.record.as_ref();
        let subject: Subjects = packet
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::OutputsCoin(subject) => Ok(OutputDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode output"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                output_index: subject.output_index.unwrap() as i64,
                output_type: "coin".to_string(),
                to_address: Some(subject.to_address.unwrap().to_string()),
                asset_id: Some(subject.asset_id.unwrap().to_string()),
                contract_id: None,
            }),
            Subjects::OutputsContract(subject) => Ok(OutputDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode output"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                output_index: subject.output_index.unwrap() as i64,
                output_type: "contract".to_string(),
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract_id.unwrap().to_string()),
            }),
            Subjects::OutputsChange(subject) => Ok(OutputDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode output"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                output_index: subject.output_index.unwrap() as i64,
                output_type: "change".to_string(),
                to_address: Some(subject.to_address.unwrap().to_string()),
                asset_id: Some(subject.asset_id.unwrap().to_string()),
                contract_id: None,
            }),
            Subjects::OutputsVariable(subject) => Ok(OutputDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode output"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                output_index: subject.output_index.unwrap() as i64,
                output_type: "variable".to_string(),
                to_address: Some(subject.to_address.unwrap().to_string()),
                asset_id: Some(subject.asset_id.unwrap().to_string()),
                contract_id: None,
            }),
            Subjects::OutputsContractCreated(subject) => Ok(OutputDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode output"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                output_index: subject.output_index.unwrap() as i64,
                output_type: "contract_created".to_string(),
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract_id.unwrap().to_string()),
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
        }
    }
}
