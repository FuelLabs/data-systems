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
pub struct ReceiptDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: BlockHeight,
    pub tx_id: String,
    pub tx_index: i32,
    pub receipt_index: i32,
    pub receipt_type: String,
    pub from_contract_id: Option<String>, // for call/transfer/transfer_out
    pub to_contract_id: Option<String>,   // for call/transfer
    pub to_address: Option<String>,       // for transfer_out
    pub asset_id: Option<String>,         // for call/transfer/transfer_out
    pub contract_id: Option<String>, /* for return/return_data/panic/revert/log/log_data/mint/burn */
    pub sub_id: Option<String>,      // for mint/burn
    pub sender_address: Option<String>, // for message_out
    pub recipient_address: Option<String>, // for message_out
    pub created_at: BlockTimestamp,
    pub published_at: BlockTimestamp,
}

impl DataEncoder for ReceiptDbItem {}

impl DbItem for ReceiptDbItem {
    fn cursor(&self) -> Cursor {
        Cursor::new(&[&self.block_height, &self.tx_index, &self.receipt_index])
    }

    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Receipt
    }

    fn encoded_value(&self) -> Result<Vec<u8>, DbError> {
        Ok(self.value.clone())
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }

    fn subject_id(&self) -> String {
        match self.receipt_type.as_str() {
            "call" => ReceiptsCallSubject::ID,
            "return" => ReceiptsReturnSubject::ID,
            "return_data" => ReceiptsReturnDataSubject::ID,
            "panic" => ReceiptsPanicSubject::ID,
            "revert" => ReceiptsRevertSubject::ID,
            "log" => ReceiptsLogSubject::ID,
            "log_data" => ReceiptsLogDataSubject::ID,
            "transfer" => ReceiptsTransferSubject::ID,
            "transfer_out" => ReceiptsTransferOutSubject::ID,
            "script_result" => ReceiptsScriptResultSubject::ID,
            "message_out" => ReceiptsMessageOutSubject::ID,
            "mint" => ReceiptsMintSubject::ID,
            "burn" => ReceiptsBurnSubject::ID,
            _ => ReceiptsSubject::ID,
        }
        .to_string()
    }

    fn created_at(&self) -> BlockTimestamp {
        self.created_at
    }

    fn block_time(&self) -> BlockTimestamp {
        self.published_at
    }

    fn block_height(&self) -> BlockHeight {
        self.block_height
    }
}

impl TryFrom<&RecordPacket> for ReceiptDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::ReceiptsCall(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "call".to_string(),
                from_contract_id: Some(subject.from.unwrap().to_string()),
                to_contract_id: Some(subject.to.unwrap().to_string()),
                asset_id: Some(subject.asset.unwrap().to_string()),
                to_address: None,
                contract_id: None,
                sub_id: None,
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsReturn(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "return".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsReturnData(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "return_data".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsPanic(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "panic".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsRevert(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "revert".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsLog(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "log".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsLogData(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "log_data".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract.unwrap().to_string()),
                sub_id: None,
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsTransfer(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "transfer".to_string(),
                from_contract_id: Some(subject.from.unwrap().to_string()),
                to_contract_id: Some(subject.to.unwrap().to_string()),
                asset_id: Some(subject.asset.unwrap().to_string()),
                contract_id: None,
                sub_id: None,
                sender_address: None,
                recipient_address: None,
                to_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsTransferOut(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "transfer_out".to_string(),
                from_contract_id: Some(subject.from.unwrap().to_string()),
                to_contract_id: None,
                to_address: Some(subject.to_address.unwrap().to_string()),
                asset_id: Some(subject.asset.unwrap().to_string()),
                contract_id: None,
                sub_id: None,
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsScriptResult(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "script_result".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: None,
                sub_id: None,
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsMessageOut(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "message_out".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: None,
                sub_id: None,
                sender_address: Some(subject.sender.unwrap().to_string()),
                recipient_address: Some(subject.recipient.unwrap().to_string()),
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsMint(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "mint".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract.unwrap().to_string()),
                sub_id: Some(subject.sub_id.unwrap().to_string()),
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            Subjects::ReceiptsBurn(subject) => Ok(ReceiptDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                receipt_index: subject.receipt_index.unwrap(),
                receipt_type: "burn".to_string(),
                from_contract_id: None,
                to_contract_id: None,
                to_address: None,
                asset_id: None,
                contract_id: Some(subject.contract.unwrap().to_string()),
                sub_id: Some(subject.sub_id.unwrap().to_string()),
                sender_address: None,
                recipient_address: None,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
        }
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

impl From<ReceiptDbItem> for RecordPointer {
    fn from(val: ReceiptDbItem) -> Self {
        RecordPointer {
            block_height: val.block_height,
            tx_index: Some(val.tx_index as u32),
            input_index: None,
            output_index: None,
            receipt_index: Some(val.receipt_index as u32),
        }
    }
}
