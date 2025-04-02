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
pub struct OutputDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: BlockHeight,
    pub tx_id: String,
    pub tx_index: i32,
    pub output_index: i32,
    pub output_type: String,
    pub to_address: Option<String>, // for coin, change, and variable outputs
    pub asset_id: Option<String>,   // for coin, change, and variable outputs
    pub contract_id: Option<String>, /* for contract and contract_created outputs */
    pub created_at: BlockTimestamp,
    pub published_at: BlockTimestamp,
}

impl DataEncoder for OutputDbItem {}

impl DbItem for OutputDbItem {
    fn cursor(&self) -> Cursor {
        Cursor::new(&[&self.block_height, &self.tx_index, &self.output_index])
    }

    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Output
    }

    fn encoded_value(&self) -> Result<Vec<u8>, DbError> {
        Ok(self.value.clone())
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }

    fn subject_id(&self) -> String {
        match self.output_type.as_str() {
            "coin" => OutputsCoinSubject::ID,
            "contract" => OutputsContractSubject::ID,
            "change" => OutputsChangeSubject::ID,
            "variable" => OutputsVariableSubject::ID,
            "contract_created" => OutputsContractCreatedSubject::ID,
            _ => OutputsSubject::ID,
        }
        .to_string()
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

impl TryFrom<&RecordPacket> for OutputDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::OutputsCoin(subject) => Ok(OutputDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i32,
                output_index: subject.output_index.unwrap() as i32,
                output_type: "coin".to_string(),
                to_address: Some(subject.to.unwrap().to_string()),
                asset_id: Some(subject.asset.unwrap().to_string()),
                contract_id: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::OutputsContract(subject) => Ok(OutputDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i32,
                output_index: subject.output_index.unwrap() as i32,
                output_type: "contract".to_string(),
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract.unwrap().to_string()),
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::OutputsChange(subject) => Ok(OutputDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i32,
                output_index: subject.output_index.unwrap() as i32,
                output_type: "change".to_string(),
                to_address: Some(subject.to.unwrap().to_string()),
                asset_id: Some(subject.asset.unwrap().to_string()),
                contract_id: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::OutputsVariable(subject) => Ok(OutputDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i32,
                output_index: subject.output_index.unwrap() as i32,
                output_type: "variable".to_string(),
                to_address: Some(subject.to.unwrap().to_string()),
                asset_id: Some(subject.asset.unwrap().to_string()),
                contract_id: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::OutputsContractCreated(subject) => Ok(OutputDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i32,
                output_index: subject.output_index.unwrap() as i32,
                output_type: "contract_created".to_string(),
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract.unwrap().to_string()),
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
        }
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

impl From<OutputDbItem> for RecordPointer {
    fn from(val: OutputDbItem) -> Self {
        RecordPointer {
            block_height: val.block_height,
            tx_index: Some(val.tx_index as u32),
            input_index: None,
            output_index: Some(val.output_index as u32),
            receipt_index: None,
        }
    }
}
