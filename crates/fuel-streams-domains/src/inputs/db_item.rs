use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, DbItem},
    record::{DataEncoder, RecordEntity, RecordPacket, RecordPacketError},
    try_packet_subject_match,
};
use serde::{Deserialize, Serialize};

use super::{subjects::*, Input};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct InputDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_id: String,
    pub tx_index: i64,
    pub input_index: i64,
    pub input_type: String,
    pub owner_id: Option<String>, // for coin inputs
    pub asset_id: Option<String>, // for coin inputs
    pub contract_id: Option<String>, // for contract inputs
    pub sender: Option<String>,   // for message inputs
    pub recipient: Option<String>, // for message inputs
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

impl TryFrom<&RecordPacket<Input>> for InputDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket<Input>) -> Result<Self, Self::Error> {
        let record = packet.record.as_ref();
        try_packet_subject_match!(packet, {
            InputsCoinSubject => subject => {
                Ok(InputDbItem {
                    subject: packet.subject_str(),
                    value: record.encode_json().expect("Failed to encode input"),
                    block_height: subject.block_height.unwrap().into(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap() as i64,
                    input_index: subject.input_index.unwrap() as i64,
                    input_type: "coin".to_string(),
                    owner_id: Some(subject.owner_id.unwrap().to_string()),
                    asset_id: Some(subject.asset_id.unwrap().to_string()),
                    contract_id: None,
                    sender: None,
                    recipient: None,
                })
            },
            InputsContractSubject => subject => {
                Ok(InputDbItem {
                    subject: packet.subject_str(),
                    value: record.encode_json().expect("Failed to encode input"),
                    block_height: subject.block_height.unwrap().into(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap() as i64,
                    input_index: subject.input_index.unwrap() as i64,
                    input_type: "contract".to_string(),
                    owner_id: None,
                    asset_id: None,
                    contract_id: Some(subject.contract_id.unwrap().to_string()),
                    sender: None,
                    recipient: None,
                })
            },
            InputsMessageSubject => subject => {
                Ok(InputDbItem {
                    subject: packet.subject_str(),
                    value: record.encode_json().expect("Failed to encode input"),
                    block_height: subject.block_height.unwrap().into(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap() as i64,
                    input_index: subject.input_index.unwrap() as i64,
                    input_type: "message".to_string(),
                    owner_id: None,
                    asset_id: None,
                    contract_id: None,
                    sender: Some(subject.sender.unwrap().to_string()),
                    recipient: Some(subject.recipient.unwrap().to_string()),
                })
            }
        })
    }
}
