use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, DbItem},
    record::{DataEncoder, RecordEntity, RecordPacket, RecordPacketError},
    try_packet_subject_match,
};
use serde::{Deserialize, Serialize};

use super::Receipt;
use crate::receipts::subjects::*;

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

        try_packet_subject_match!(packet, {
            ReceiptsCallSubject => subject => {
                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: record.encode_json().expect("Failed to encode receipt"),
                    block_height: subject.block_height.unwrap().into(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap() as i64,
                    receipt_index: subject.receipt_index.unwrap() as i64,
                    receipt_type: "call".to_string(),
                    from_contract_id: Some(subject.from.unwrap().to_string()),
                    to_contract_id: Some(subject.to.unwrap().to_string()),
                    to_address: None,
                    asset_id: Some(subject.asset_id.unwrap().to_string()),
                    contract_id: None,
                    sub_id: None,
                    sender_address: None,
                    recipient_address: None,
                })
            },
            ReceiptsReturnSubject => subject => {
                Ok(ReceiptDbItem {
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
                    contract_id: Some(subject.id.unwrap().to_string()),
                    sub_id: None,
                    sender_address: None,
                    recipient_address: None,
                })
            },
            ReceiptsReturnDataSubject => subject => {
                Ok(ReceiptDbItem {
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
                    contract_id: Some(subject.id.unwrap().to_string()),
                    sub_id: None,
                    sender_address: None,
                    recipient_address: None,
                })
            },
            ReceiptsPanicSubject => subject => {
                Ok(ReceiptDbItem {
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
                    contract_id: Some(subject.id.unwrap().to_string()),
                    sub_id: None,
                    sender_address: None,
                    recipient_address: None,
                })
            },
            ReceiptsRevertSubject => subject => {
                Ok(ReceiptDbItem {
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
                    contract_id: Some(subject.id.unwrap().to_string()),
                    sub_id: None,
                    sender_address: None,
                    recipient_address: None,
                })
            },
            ReceiptsLogSubject => subject => {
                Ok(ReceiptDbItem {
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
                    contract_id: Some(subject.id.unwrap().to_string()),
                    sub_id: None,
                    sender_address: None,
                    recipient_address: None,
                })
            },
            ReceiptsLogDataSubject => subject => {
                Ok(ReceiptDbItem {
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
                    contract_id: Some(subject.id.unwrap().to_string()),
                    sub_id: None,
                    sender_address: None,
                    recipient_address: None,
                })
            },
            ReceiptsTransferSubject => subject => {
                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: record.encode_json().expect("Failed to encode receipt"),
                    block_height: subject.block_height.unwrap().into(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap() as i64,
                    receipt_index: subject.receipt_index.unwrap() as i64,
                    receipt_type: "transfer".to_string(),
                    from_contract_id: Some(subject.from.unwrap().to_string()),
                    to_contract_id: Some(subject.to.unwrap().to_string()),
                    to_address: None,
                    asset_id: Some(subject.asset_id.unwrap().to_string()),
                    contract_id: None,
                    sub_id: None,
                    sender_address: None,
                    recipient_address: None,
                })
            },
            ReceiptsTransferOutSubject => subject => {
                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: record.encode_json().expect("Failed to encode receipt"),
                    block_height: subject.block_height.unwrap().into(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap() as i64,
                    receipt_index: subject.receipt_index.unwrap() as i64,
                    receipt_type: "transfer_out".to_string(),
                    from_contract_id: Some(subject.from.unwrap().to_string()),
                    to_contract_id: None,
                    to_address: Some(subject.to.unwrap().to_string()),
                    asset_id: Some(subject.asset_id.unwrap().to_string()),
                    contract_id: None,
                    sub_id: None,
                    sender_address: None,
                    recipient_address: None,
                })
            },
            ReceiptsScriptResultSubject => subject => {
                Ok(ReceiptDbItem {
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
                })
            },
            ReceiptsMessageOutSubject => subject => {
                Ok(ReceiptDbItem {
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
                    sender_address: Some(subject.sender.unwrap().to_string()),
                    recipient_address: Some(subject.recipient.unwrap().to_string()),
                })
            },
            ReceiptsMintSubject => subject => {
                Ok(ReceiptDbItem {
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
                })
            },
            ReceiptsBurnSubject => subject => {
                Ok(ReceiptDbItem {
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
                })
            },
        })
    }
}
