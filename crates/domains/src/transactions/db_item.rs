use std::cmp::Ordering;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::{
    Amount,
    BlockHeight,
    BlockTimestamp,
    GasAmount,
    TransactionStatus,
    TransactionType,
    TxId,
    TxPointer,
};
use serde::{Deserialize, Serialize};

use super::subjects::*;
use crate::{
    infra::{
        Cursor,
        DbError,
        DbItem,
        RecordEntity,
        RecordPacket,
        RecordPacketError,
        RecordPointer,
    },
    transactions::Transaction,
    Subjects,
};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct TransactionDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: BlockHeight,
    pub tx_id: TxId,
    pub tx_index: i32,
    pub r#type: TransactionType,
    pub script_gas_limit: Option<GasAmount>,
    pub mint_amount: Option<Amount>,
    pub mint_asset_id: Option<String>,
    pub mint_gas_price: Option<Amount>,
    pub receipts_root: Option<String>,
    pub status: TransactionStatus,
    pub script: Option<String>,
    pub script_data: Option<String>,
    pub salt: Option<String>,
    pub bytecode_witness_index: Option<i32>,
    pub bytecode_root: Option<String>,
    pub subsection_index: Option<i32>,
    pub subsections_number: Option<i32>,
    pub upgrade_purpose: Option<String>,
    pub blob_id: Option<String>,
    pub is_blob: bool,
    pub is_create: bool,
    pub is_mint: bool,
    pub is_script: bool,
    pub is_upgrade: bool,
    pub is_upload: bool,
    pub raw_payload: String,
    pub tx_pointer: Option<Vec<u8>>,
    pub maturity: Option<i32>,
    pub script_length: Option<i32>,
    pub script_data_length: Option<i32>,
    pub storage_slots_count: Option<i32>,
    pub proof_set_count: Option<i32>,
    pub witnesses_count: Option<i32>,
    pub inputs_count: Option<i32>,
    pub outputs_count: Option<i32>,
    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
}

impl DataEncoder for TransactionDbItem {}

impl DbItem for TransactionDbItem {
    fn cursor(&self) -> crate::infra::Cursor {
        Cursor::new(&[&self.block_height, &self.tx_index])
    }

    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Transaction
    }

    fn encoded_value(&self) -> Result<Vec<u8>, DbError> {
        Ok(self.value.to_owned())
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }

    fn subject_id(&self) -> String {
        TransactionsSubject::ID.to_string()
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

impl TryFrom<&RecordPacket> for TransactionDbItem {
    type Error = RecordPacketError;

    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let transaction = Transaction::decode_json(&packet.value)?;
        let tx_pointer = match transaction.tx_pointer {
            Some(tx_pointer) => Some(serde_json::to_vec(&tx_pointer)?),
            None => Some(serde_json::to_vec(&TxPointer {
                block_height: packet.pointer.block_height.to_owned(),
                tx_index: packet
                    .pointer
                    .tx_index
                    .expect("tx_index should be defined")
                    as u16,
            })?),
        };

        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::Transactions(_) => Ok(TransactionDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: packet.pointer.block_height,
                tx_id: packet.pointer.tx_id.to_owned().unwrap(),
                tx_index: packet.pointer.tx_index.unwrap() as i32,
                r#type: transaction.r#type,
                status: transaction.status,
                script_gas_limit: transaction.script_gas_limit,
                mint_amount: transaction.mint_amount,
                mint_asset_id: transaction
                    .mint_asset_id
                    .map(|id| id.to_string()),
                mint_gas_price: transaction.mint_gas_price,
                receipts_root: transaction
                    .receipts_root
                    .map(|root| root.to_string()),
                script: transaction
                    .script
                    .as_ref()
                    .map(|script| script.to_string()),
                script_data: transaction
                    .script_data
                    .as_ref()
                    .map(|data| data.to_string()),
                salt: transaction.salt.map(|salt| salt.to_string()),
                bytecode_witness_index: transaction
                    .bytecode_witness_index
                    .map(|i| i as i32),
                bytecode_root: transaction
                    .bytecode_root
                    .map(|root| root.to_string()),
                subsection_index: transaction
                    .subsection_index
                    .map(|i| i as i32),
                subsections_number: transaction
                    .subsections_number
                    .map(|n| n as i32),
                upgrade_purpose: transaction
                    .upgrade_purpose
                    .map(|p| p.to_string()),
                blob_id: transaction.blob_id.map(|id| id.to_string()),
                is_blob: transaction.is_blob,
                is_create: transaction.is_create,
                is_mint: transaction.is_mint,
                is_script: transaction.is_script,
                is_upgrade: transaction.is_upgrade,
                is_upload: transaction.is_upload,
                raw_payload: transaction.raw_payload.to_string(),
                maturity: transaction.maturity.map(|m| m as i32),
                tx_pointer,
                script_length: transaction.script_length.map(|l| l as i32),
                script_data_length: transaction
                    .script_data_length
                    .map(|l| l as i32),
                storage_slots_count: Some(
                    transaction.storage_slots_count as i32,
                ),
                proof_set_count: Some(transaction.proof_set_count as i32),
                witnesses_count: Some(transaction.witnesses_count as i32),
                inputs_count: Some(transaction.inputs_count as i32),
                outputs_count: Some(transaction.outputs_count as i32),
                block_time: packet.block_timestamp,
                created_at: packet.block_timestamp,
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
        }
    }
}

impl PartialOrd for TransactionDbItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TransactionDbItem {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.block_height.cmp(&other.block_height) {
            Ordering::Equal => self.tx_index.cmp(&other.tx_index),
            ord => ord,
        }
    }
}

impl From<TransactionDbItem> for RecordPointer {
    fn from(val: TransactionDbItem) -> Self {
        RecordPointer {
            block_height: val.block_height,
            tx_id: Some(val.tx_id),
            tx_index: Some(val.tx_index as u32),
            input_index: None,
            output_index: None,
            receipt_index: None,
        }
    }
}
