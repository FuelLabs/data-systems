use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, DbItem},
    record::{DataEncoder, RecordEntity, RecordPacket, RecordPacketError},
};
use fuel_streams_types::BlockHeight;
use serde::{Deserialize, Serialize};

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

    fn get_block_height(&self) -> BlockHeight {
        self.block_height.into()
    }
}

impl TryFrom<&RecordPacket> for InputDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
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
