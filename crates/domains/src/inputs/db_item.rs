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
use fuel_streams_types::BlockTimestamp;
use serde::{Deserialize, Serialize};

use super::subjects::*;
use crate::Subjects;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct InputDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_id: String,
    pub tx_index: i32,
    pub input_index: i32,
    pub input_type: String,
    pub owner_id: Option<String>, // for coin inputs
    pub asset_id: Option<String>, // for coin inputs
    pub contract_id: Option<String>, // for contract inputs
    pub sender_address: Option<String>, // for message inputs
    pub recipient_address: Option<String>, // for message inputs
    pub created_at: BlockTimestamp,
    pub published_at: BlockTimestamp,
}

impl DataEncoder for InputDbItem {
    type Err = DbError;
}

impl DbItem for InputDbItem {
    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Input
    }

    fn encoded_value(&self) -> &[u8] {
        &self.value
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }

    fn subject_id(&self) -> String {
        match self.input_type.as_str() {
            "coin" => InputsCoinSubject::ID,
            "contract" => InputsContractSubject::ID,
            "message" => InputsMessageSubject::ID,
            _ => InputsSubject::ID,
        }
        .to_string()
    }

    fn created_at(&self) -> BlockTimestamp {
        self.created_at
    }

    fn published_at(&self) -> BlockTimestamp {
        self.published_at
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
            Subjects::InputsCoin(subject) => Ok(InputDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i32,
                input_index: subject.input_index.unwrap() as i32,
                input_type: "coin".to_string(),
                owner_id: Some(subject.owner.unwrap().to_string()),
                asset_id: Some(subject.asset.unwrap().to_string()),
                contract_id: None,
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::InputsContract(subject) => Ok(InputDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i32,
                input_index: subject.input_index.unwrap() as i32,
                input_type: "contract".to_string(),
                owner_id: None,
                asset_id: None,
                contract_id: Some(subject.contract.unwrap().to_string()),
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::InputsMessage(subject) => Ok(InputDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i32,
                input_index: subject.input_index.unwrap() as i32,
                input_type: "message".to_string(),
                owner_id: None,
                asset_id: None,
                contract_id: None,
                sender_address: Some(subject.sender.unwrap().to_string()),
                recipient_address: Some(subject.recipient.unwrap().to_string()),
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
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
            block_height: val.block_height.into(),
            tx_index: Some(val.tx_index as u32),
            input_index: Some(val.input_index as u32),
            output_index: None,
            receipt_index: None,
        }
    }
}
