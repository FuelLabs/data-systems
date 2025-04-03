use fuel_streams_types::{BlockHeight, BlockTimestamp};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgHasArrayType, PgTypeInfo};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
#[sqlx(type_name = "transaction_storage_slots")]
pub struct TransactionStorageSlotDbItem {
    pub tx_id: String,
    pub block_height: BlockHeight,
    pub key: String,
    pub value: String,
    pub block_time: BlockTimestamp,
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

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
#[sqlx(type_name = "transaction_witnesses")]
pub struct TransactionWitnessDbItem {
    pub tx_id: String,
    pub block_height: BlockHeight,
    pub witness_data: String,
    pub witness_data_length: i32,
    pub block_time: BlockTimestamp,
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

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
#[sqlx(type_name = "transaction_proof_set")]
pub struct TransactionProofSetDbItem {
    pub tx_id: String,
    pub block_height: BlockHeight,
    pub proof_hash: String,
    pub block_time: BlockTimestamp,
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

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
#[sqlx(type_name = "transaction_policies")]
pub struct TransactionPolicyDbItem {
    pub tx_id: String,
    pub block_height: BlockHeight,
    pub tip: Option<i64>,
    pub maturity: Option<i32>,
    pub witness_limit: Option<i64>,
    pub max_fee: Option<i64>,
    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
}

impl sqlx::Type<sqlx::Postgres> for TransactionPolicyDbItem {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("transaction_policies")
    }
}

impl PgHasArrayType for TransactionPolicyDbItem {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_transaction_policies")
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
#[sqlx(type_name = "transaction_input_contracts")]
pub struct TransactionInputContractDbItem {
    pub tx_id: String,
    pub block_height: BlockHeight,
    pub contract_id: String,
    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
}

impl sqlx::Type<sqlx::Postgres> for TransactionInputContractDbItem {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("transaction_input_contracts")
    }
}

impl PgHasArrayType for TransactionInputContractDbItem {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_transaction_input_contracts")
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
#[sqlx(type_name = "transaction_input_contract")]
pub struct TransactionInputContractSingleDbItem {
    pub tx_id: String,
    pub block_height: BlockHeight,
    pub balance_root: String,
    pub contract_id: String,
    pub state_root: String,
    pub tx_pointer: Vec<u8>,
    pub utxo_id: String,
    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
}

impl sqlx::Type<sqlx::Postgres> for TransactionInputContractSingleDbItem {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("transaction_input_contract")
    }
}

impl PgHasArrayType for TransactionInputContractSingleDbItem {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_transaction_input_contract")
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
#[sqlx(type_name = "transaction_output_contract")]
pub struct TransactionOutputContractDbItem {
    pub tx_id: String,
    pub block_height: BlockHeight,
    pub balance_root: String,
    pub input_index: i32,
    pub state_root: String,
    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
}

impl sqlx::Type<sqlx::Postgres> for TransactionOutputContractDbItem {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("transaction_output_contract")
    }
}

impl PgHasArrayType for TransactionOutputContractDbItem {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_transaction_output_contract")
    }
}
