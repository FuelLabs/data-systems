use std::cmp::Ordering;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::{subjects::*, Output};
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

    // Common props
    pub r#type: OutputType,

    // coin/change/variable shared props
    pub amount: Option<i64>,
    pub asset_id: Option<String>,
    pub to_address: Option<String>,

    // contract/contract_created shared props
    pub state_root: Option<String>,

    // contract specific props
    pub balance_root: Option<String>,
    pub input_index: Option<i32>,

    // contract_created specific props
    pub contract_id: Option<String>,

    // timestamps
    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
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
        match self.r#type {
            OutputType::Coin => OutputsCoinSubject::ID,
            OutputType::Contract => OutputsContractSubject::ID,
            OutputType::Change => OutputsChangeSubject::ID,
            OutputType::Variable => OutputsVariableSubject::ID,
            OutputType::ContractCreated => OutputsContractCreatedSubject::ID,
        }
        .to_string()
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

impl TryFrom<&RecordPacket> for OutputDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::OutputsCoin(subject) => {
                let output = match Output::decode_json(&packet.value)? {
                    Output::Coin(coin) => coin,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(OutputDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    output_index: subject.output_index.unwrap(),
                    r#type: OutputType::Coin,
                    amount: Some(output.amount.into_inner() as i64),
                    asset_id: Some(output.asset_id.to_string()),
                    to_address: Some(output.to.to_string()),
                    state_root: None,
                    balance_root: None,
                    input_index: None,
                    contract_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::OutputsContract(subject) => {
                let output = match Output::decode_json(&packet.value)? {
                    Output::Contract(contract) => contract,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(OutputDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    output_index: subject.output_index.unwrap(),
                    r#type: OutputType::Contract,
                    amount: None,
                    asset_id: None,
                    to_address: None,
                    state_root: Some(output.state_root.to_string()),
                    balance_root: Some(output.balance_root.to_string()),
                    input_index: Some(output.input_index as i32),
                    contract_id: Some(subject.contract.unwrap().to_string()),
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::OutputsChange(subject) => {
                let output = match Output::decode_json(&packet.value)? {
                    Output::Change(change) => change,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(OutputDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    output_index: subject.output_index.unwrap(),
                    r#type: OutputType::Change,
                    amount: Some(output.amount.into_inner() as i64),
                    asset_id: Some(output.asset_id.to_string()),
                    to_address: Some(output.to.to_string()),
                    state_root: None,
                    balance_root: None,
                    input_index: None,
                    contract_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::OutputsVariable(subject) => {
                let output = match Output::decode_json(&packet.value)? {
                    Output::Variable(variable) => variable,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(OutputDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    output_index: subject.output_index.unwrap(),
                    r#type: OutputType::Variable,
                    amount: Some(output.amount.into_inner() as i64),
                    asset_id: Some(output.asset_id.to_string()),
                    to_address: Some(output.to.to_string()),
                    state_root: None,
                    balance_root: None,
                    input_index: None,
                    contract_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::OutputsContractCreated(subject) => {
                let output = match Output::decode_json(&packet.value)? {
                    Output::ContractCreated(contract) => contract,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(OutputDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    output_index: subject.output_index.unwrap(),
                    r#type: OutputType::ContractCreated,
                    amount: None,
                    asset_id: None,
                    to_address: None,
                    state_root: Some(output.state_root.to_string()),
                    balance_root: None,
                    input_index: None,
                    contract_id: Some(output.contract_id.to_string()),
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
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
        self.block_height
            .cmp(&other.block_height)
            .then(self.tx_index.cmp(&other.tx_index))
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
