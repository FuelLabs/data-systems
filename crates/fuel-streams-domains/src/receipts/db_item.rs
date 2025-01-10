use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, DbItem},
    record::{DataEncoder, RecordEntity, RecordPacket, RecordPacketError},
};
use serde::{Deserialize, Serialize};

use super::Receipt;
use crate::Subjects;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct ReceiptDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_id: String,
    pub tx_index: i64,
    pub receipt_index: i64,
    pub receipt_type: String,
    pub from_contract_id: Option<String>, // for call/transfer/transfer_out
    pub to_contract_id: Option<String>,   // for call/transfer
    pub to_address: Option<String>,       // for transfer_out
    pub asset_id: Option<String>,         // for call/transfer/transfer_out
    pub contract_id: Option<String>, /* for return/return_data/panic/revert/log/log_data/mint/burn */
    pub sub_id: Option<String>,      // for mint/burn
    pub sender_address: Option<String>, // for message_out
    pub recipient_address: Option<String>, // for message_out
}

impl DataEncoder for ReceiptDbItem {
    type Err = DbError;
}

impl DbItem for ReceiptDbItem {
    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Receipt
    }

    fn encoded_value(&self) -> &[u8] {
        &self.value
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }
}

impl PartialOrd for ReceiptDbItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ReceiptDbItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order by block height first
        self.block_height
            .cmp(&other.block_height)
            // Then by transaction index within the block
            .then(self.tx_index.cmp(&other.tx_index))
            // Finally by receipt index within the transaction
            .then(self.receipt_index.cmp(&other.receipt_index))
    }
}

impl TryFrom<&RecordPacket<Receipt>> for ReceiptDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket<Receipt>) -> Result<Self, Self::Error> {
        let record = packet.record.as_ref();
        let subject: Subjects = packet
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::ReceiptsCall(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "call".to_string(),
                from_contract_id: Some(
                    subject.from_contract_id.unwrap().to_string(),
                ),
                to_contract_id: Some(
                    subject.to_contract_id.unwrap().to_string(),
                ),
                asset_id: Some(subject.asset_id.unwrap().to_string()),
                to_address: None,
                contract_id: None,
                sub_id: None,
                sender_address: None,
                recipient_address: None,
            }),
            Subjects::ReceiptsReturn(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "return".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract_id.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
            }),
            Subjects::ReceiptsReturnData(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "return_data".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract_id.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
            }),
            Subjects::ReceiptsPanic(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "panic".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract_id.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
            }),
            Subjects::ReceiptsRevert(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "revert".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract_id.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
            }),
            Subjects::ReceiptsLog(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "log".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract_id.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
            }),
            Subjects::ReceiptsLogData(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "log_data".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract_id.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
            }),
            Subjects::ReceiptsTransfer(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "transfer".to_string(),
                from_contract_id: Some(
                    subject.from_contract_id.unwrap().to_string(),
                ),
                to_contract_id: Some(
                    subject.to_contract_id.unwrap().to_string(),
                ),
                asset_id: Some(subject.asset_id.unwrap().to_string()),
                contract_id: None,
                sub_id: None,
                sender_address: None,
                recipient_address: None,
                to_address: None,
            }),
            Subjects::ReceiptsTransferOut(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "transfer_out".to_string(),
                from_contract_id: Some(
                    subject.from_contract_id.unwrap().to_string(),
                ),
                to_contract_id: None,
                to_address: Some(subject.to_address.unwrap().to_string()),
                asset_id: Some(subject.asset_id.unwrap().to_string()),
                contract_id: None,
                sub_id: None,
                sender_address: None,
                recipient_address: None,
            }),
            Subjects::ReceiptsScriptResult(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "script_result".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: None,
                sub_id: None,
                sender_address: None,
                recipient_address: None,
            }),
            Subjects::ReceiptsMessageOut(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "message_out".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: None,
                sub_id: None,
                sender_address: Some(
                    subject.sender_address.unwrap().to_string(),
                ),
                recipient_address: Some(
                    subject.recipient_address.unwrap().to_string(),
                ),
            }),
            Subjects::ReceiptsMint(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "mint".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract_id.unwrap().to_string()),
                sub_id: Some(subject.sub_id.unwrap().to_string()),
                sender_address: None,
                recipient_address: None,
            }),
            Subjects::ReceiptsBurn(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode receipt"),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i64,
                receipt_index: subject.receipt_index.unwrap() as i64,
                receipt_type: "burn".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract_id.unwrap().to_string()),
                sub_id: Some(subject.sub_id.unwrap().to_string()),
                sender_address: None,
                recipient_address: None,
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
        }
    }
}
