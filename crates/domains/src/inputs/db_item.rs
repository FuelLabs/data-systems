use std::cmp::Ordering;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::{BlockHeight, BlockTimestamp, InputType};
use serde::{Deserialize, Serialize};

use super::{subjects::*, Input};
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
pub struct InputDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: BlockHeight,
    pub tx_id: String,
    pub tx_index: i32,
    pub input_index: i32,

    // Common props
    pub r#type: InputType,
    pub utxo_id: Option<String>,

    // Coin specific props
    pub amount: Option<i64>,
    pub asset_id: Option<String>,
    pub owner_id: Option<String>,

    // Contract specific props
    pub balance_root: Option<String>,
    pub contract_id: Option<String>,
    pub state_root: Option<String>,
    pub tx_pointer: Option<Vec<u8>>,

    // Message specific props
    pub sender_address: Option<String>,
    pub recipient_address: Option<String>,
    pub nonce: Option<String>,
    pub data: Option<String>,
    pub data_length: Option<i32>,

    // Predicate related props
    pub witness_index: Option<i32>,
    pub predicate_gas_used: Option<i64>,
    pub predicate: Option<String>,
    pub predicate_data: Option<String>,
    pub predicate_length: Option<i32>,
    pub predicate_data_length: Option<i32>,

    // Timestamps
    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
}

impl DataEncoder for InputDbItem {}

impl DbItem for InputDbItem {
    fn cursor(&self) -> Cursor {
        Cursor::new(&[&self.block_height, &self.tx_index, &self.input_index])
    }

    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Input
    }

    fn encoded_value(&self) -> Result<Vec<u8>, DbError> {
        Ok(self.value.clone())
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }

    fn subject_id(&self) -> String {
        match self.r#type {
            InputType::Coin => InputsCoinSubject::ID,
            InputType::Contract => InputsContractSubject::ID,
            InputType::Message => InputsMessageSubject::ID,
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

impl TryFrom<&RecordPacket> for InputDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::InputsCoin(subject) => {
                let input = match Input::decode_json(&packet.value)? {
                    Input::Coin(coin) => coin,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(InputDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    input_index: subject.input_index.unwrap(),
                    r#type: InputType::Coin,
                    utxo_id: Some(input.utxo_id.to_string()),
                    amount: Some(input.amount.into_inner() as i64),
                    asset_id: Some(input.asset_id.to_string()),
                    owner_id: Some(input.owner.to_string()),
                    balance_root: None,
                    contract_id: None,
                    state_root: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    data: None,
                    data_length: None,
                    tx_pointer: None,
                    witness_index: Some(input.witness_index as i32),
                    predicate_gas_used: Some(
                        input.predicate_gas_used.into_inner() as i64,
                    ),
                    predicate: Some(input.predicate.to_string()),
                    predicate_data: Some(input.predicate_data.to_string()),
                    predicate_length: Some(input.predicate.len() as i32),
                    predicate_data_length: Some(
                        input.predicate_data.len() as i32
                    ),
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::InputsContract(subject) => {
                let input = match Input::decode_json(&packet.value)? {
                    Input::Contract(contract) => contract,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(InputDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    input_index: subject.input_index.unwrap(),
                    r#type: InputType::Contract,
                    utxo_id: Some(input.utxo_id.to_string()),
                    balance_root: Some(input.balance_root.to_string()),
                    contract_id: Some(input.contract_id.to_string()),
                    state_root: Some(input.state_root.to_string()),
                    tx_pointer: Some(serde_json::to_vec(&input.tx_pointer)?),
                    amount: None,
                    asset_id: None,
                    owner_id: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    data: None,
                    data_length: None,
                    witness_index: None,
                    predicate_gas_used: None,
                    predicate: None,
                    predicate_data: None,
                    predicate_length: None,
                    predicate_data_length: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::InputsMessage(subject) => {
                let input = match Input::decode_json(&packet.value)? {
                    Input::Message(message) => message,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(InputDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    input_index: subject.input_index.unwrap(),
                    r#type: InputType::Message,
                    utxo_id: None,
                    amount: Some(input.amount.into_inner() as i64),
                    asset_id: None,
                    owner_id: None,
                    balance_root: None,
                    contract_id: None,
                    state_root: None,
                    tx_pointer: None,
                    sender_address: Some(input.sender.to_string()),
                    recipient_address: Some(input.recipient.to_string()),
                    nonce: Some(input.nonce.to_string()),
                    data: Some(input.data.to_string()),
                    data_length: Some(input.data.into_inner().len() as i32),
                    witness_index: Some(input.witness_index as i32),
                    predicate_gas_used: Some(
                        input.predicate_gas_used.into_inner() as i64,
                    ),
                    predicate: Some(input.predicate.to_string()),
                    predicate_data: Some(input.predicate_data.to_string()),
                    predicate_length: Some(input.predicate_length as i32),
                    predicate_data_length: Some(
                        input.predicate_data_length as i32,
                    ),
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            _ => Err(RecordPacketError::SubjectMismatch),
        }
    }
}

impl PartialOrd for InputDbItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InputDbItem {
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

impl From<InputDbItem> for RecordPointer {
    fn from(val: InputDbItem) -> Self {
        RecordPointer {
            block_height: val.block_height,
            tx_index: Some(val.tx_index as u32),
            input_index: Some(val.input_index as u32),
            output_index: None,
            receipt_index: None,
        }
    }
}
