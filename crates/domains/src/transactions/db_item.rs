use std::cmp::Ordering;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::{
    Amount,
    BlockHeight,
    BlockTimestamp,
    GasAmount,
    TransactionStatus,
    TransactionType,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgHasArrayType, PgTypeInfo};

use super::{subjects::*, PolicyWrapper};
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

// TransactionStorageSlotDbItem for saving in repository
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
#[sqlx(type_name = "transaction_storage_slots")]
pub struct TransactionStorageSlotDbItem {
    pub tx_id: String,
    pub key: String,
    pub value: String,
    pub created_at: BlockTimestamp,
}

impl sqlx::Type<sqlx::Postgres> for TransactionStorageSlotDbItem {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("transaction_storage_slots")
    }
}

impl PgHasArrayType for TransactionStorageSlotDbItem {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_transaction_storage_slots")
    }
}

// TransactionWitnessDbItem for saving in repository
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
#[sqlx(type_name = "transaction_witnesses")]
pub struct TransactionWitnessDbItem {
    pub tx_id: String,
    pub witness_data: String,
    pub witness_data_length: i32,
    pub created_at: BlockTimestamp,
}

impl sqlx::Type<sqlx::Postgres> for TransactionWitnessDbItem {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("transaction_witnesses")
    }
}

impl PgHasArrayType for TransactionWitnessDbItem {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_transaction_witnesses")
    }
}

// TransactionProofSetDbItem for saving in repository
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
#[sqlx(type_name = "transaction_proof_set")]
pub struct TransactionProofSetDbItem {
    pub tx_id: String,
    pub proof_hash: String,
    pub created_at: BlockTimestamp,
}

impl sqlx::Type<sqlx::Postgres> for TransactionProofSetDbItem {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("transaction_proof_set")
    }
}

impl PgHasArrayType for TransactionProofSetDbItem {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_transaction_proof_set")
    }
}

// Main transaction DB item without relational fields
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct TransactionDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_id: String,
    pub tx_index: i32,
    // fuel-core fields
    pub r#type: TransactionType,
    pub script_gas_limit: Option<GasAmount>,
    pub mint_amount: Option<Amount>,
    pub mint_asset_id: Option<String>,
    pub mint_gas_price: Option<Amount>,
    pub receipts_root: Option<String>,
    pub tx_status: TransactionStatus,
    pub script: Option<String>,
    pub script_data: Option<String>,
    pub salt: Option<String>,
    pub bytecode_witness_index: Option<i32>,
    pub bytecode_root: Option<String>,
    pub subsection_index: Option<i32>,
    pub subsections_number: Option<i32>,
    pub upgrade_purpose: Option<String>,
    pub blob_id: Option<String>,
    // extra fields (not in fuel-core)
    pub maturity: Option<i32>,
    pub policies: Option<String>,
    pub script_length: Option<i64>,
    pub script_data_length: Option<i64>,
    pub storage_slots_count: Option<i64>,
    pub proof_set_count: Option<i32>,
    pub witnesses_count: Option<i32>,
    pub inputs_count: Option<i32>,
    pub outputs_count: Option<i32>,
    // timestamps
    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
    pub published_at: BlockTimestamp,
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
        Ok(self.value.clone())
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
        self.published_at
    }

    fn block_height(&self) -> BlockHeight {
        self.block_height.into()
    }
}

impl TryFrom<&RecordPacket> for TransactionDbItem {
    type Error = RecordPacketError;

    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        let transaction = Transaction::decode_json(&packet.value)
            .map_err(|e| RecordPacketError::DecodeFailed(e.to_string()))?;

        let policies: Option<String> = transaction
            .policies
            .as_ref()
            .and_then(|p| p.to_owned().try_into().ok())
            .map(|wrapper: PolicyWrapper| wrapper.to_string().ok())
            .flatten();

        match subject {
            Subjects::Transactions(subject) => Ok(TransactionDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap(),
                r#type: transaction.r#type,
                tx_status: transaction.status,
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
                maturity: transaction.maturity.map(|m| m as i32),
                policies,
                script_length: transaction
                    .script
                    .as_ref()
                    .map(|s| s.0 .0.len() as i64),
                script_data_length: transaction
                    .script_data
                    .as_ref()
                    .map(|d| d.0 .0.len() as i64),
                storage_slots_count: Some(
                    transaction.storage_slots.len() as i64
                ),
                proof_set_count: Some(transaction.proof_set.len() as i32),
                witnesses_count: Some(transaction.witnesses.len() as i32),
                inputs_count: Some(transaction.inputs.len() as i32),
                outputs_count: Some(transaction.outputs.len() as i32),
                block_time: packet.block_timestamp,
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
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
            block_height: val.block_height.into(),
            tx_index: Some(val.tx_index as u32),
            input_index: None,
            output_index: None,
            receipt_index: None,
        }
    }
}
