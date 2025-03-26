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
use fuel_streams_types::{
    Amount,
    BlockHeight,
    BlockTimestamp,
    DbPolicyType,
    DbTransactionAccountType,
    DbTransactionStatus,
    DbTransactionType,
    GasAmount,
};
use fuel_tx::UpgradePurpose;
use serde::{Deserialize, Serialize};

use super::subjects::*;
use crate::{transactions::Transaction, Subjects};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct TransactionDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_id: String,
    pub tx_index: i32,
    // cursor
    pub cursor: String,
    // fuel-core fields
    pub transaction_type: DbTransactionType,
    pub script_gas_limit: Option<GasAmount>,
    pub is_create: bool,
    pub is_mint: bool,
    pub is_script: bool,
    pub is_upgrade: bool,
    pub is_upload: bool,
    pub is_blob: bool,
    pub mint_amount: Option<Amount>,
    pub mint_asset_id: Option<String>,
    pub mint_gas_price: Option<Amount>,
    pub receipts_root: Option<String>,
    pub tx_status: DbTransactionStatus,
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
    pub maturity: Option<i64>,
    pub policy_type: Option<i64>,
    pub script_length: Option<i64>,
    pub script_data_length: Option<i64>,
    pub storage_slots_count: Option<i64>,
    pub proof_set_count: Option<i64>,
    pub witnesses_count: Option<i64>,
    pub inputs_count: Option<i64>,
    pub outputs_count: Option<i64>,
    // from transactions_data
    pub transaction_data: Vec<u8>,
    // from transaction_storage_slots
    pub transaction_storage_slots_key: Vec<String>,
    pub transaction_storage_slots_value: Vec<String>,
    // from transaction_witnesses
    pub witness_data: Vec<String>,
    pub witness_data_length: Vec<i64>,
    // from transaction_proof_set
    pub transaction_proof_set_proof_hash: Vec<String>,
    // from transaction_policies
    pub transaction_policy_type: Vec<DbPolicyType>,
    pub transaction_policy_data: Vec<String>,
    // from transaction_accounts
    pub transaction_account_address: Vec<String>,
    pub transaction_account_type: Vec<DbTransactionAccountType>,
    // timestamps
    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
    pub published_at: BlockTimestamp,
}

impl DataEncoder for TransactionDbItem {
    type Err = DbError;
}

impl DbItem for TransactionDbItem {
    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Transaction
    }

    fn encoded_value(&self) -> &[u8] {
        &self.value
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

    fn published_at(&self) -> BlockTimestamp {
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

        match subject {
            Subjects::Transactions(subject) => Ok(TransactionDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.block_height.unwrap().into(),
                tx_id: subject.tx_id.unwrap().to_string(),
                tx_index: subject.tx_index.unwrap() as i32,
                cursor: format!(
                    "{}-{}",
                    subject.block_height.unwrap(),
                    subject.tx_index.unwrap()
                ),
                transaction_type: (&transaction.tx_type).into(),
                // -- fields matching fuel-core
                script_gas_limit: transaction.script_gas_limit,
                is_create: transaction.is_create,
                is_mint: transaction.is_mint,
                is_script: transaction.is_script,
                is_upgrade: transaction.is_upgrade,
                is_upload: transaction.is_upload,
                is_blob: transaction.is_blob(),
                mint_amount: transaction.mint_amount,
                mint_asset_id: transaction
                    .mint_asset_id
                    .map(|mint_asset_id| mint_asset_id.to_string()),
                mint_gas_price: transaction.mint_gas_price,
                receipts_root: transaction
                    .receipts_root
                    .map(|receipts_root| receipts_root.to_string()),
                tx_status: (&transaction.status).into(),
                script: transaction.script.map(|script| script.to_string()),
                script_data: transaction
                    .script_data
                    .map(|data| data.to_string()),
                salt: transaction.salt.map(|salt| salt.to_string()),
                bytecode_witness_index: transaction.bytecode_witness_index.map(
                    |bytecode_witness_index| bytecode_witness_index as i32,
                ),
                bytecode_root: transaction
                    .bytecode_root
                    .map(|bytecode_root| bytecode_root.to_string()),
                subsection_index: transaction
                    .subsection_index
                    .map(|subsection_index| subsection_index as i32),
                subsections_number: transaction
                    .subsections_number
                    .map(|subsections_number| subsections_number as i32),
                upgrade_purpose: transaction.upgrade_purpose.map(
                    |upgrade_purpose| match upgrade_purpose.0 {
                        UpgradePurpose::ConsensusParameters {
                            checksum,
                            ..
                        } => checksum.to_string(),
                        UpgradePurpose::StateTransition { root, .. } => {
                            root.to_string()
                        }
                    },
                ),
                blob_id: transaction.blob_id.map(|blob_id| blob_id.to_string()),
                // -- extra fields (not in fuel-core)
                maturity: transaction.maturity.map(|maturity| maturity as i64),
                policy_type: transaction
                    .policies
                    .map(|policy_type| policy_type.0.bits() as i64),
                script_length: transaction
                    .script
                    .map(|script_length| script_length.0 .0.len() as i64),
                script_data_length: transaction.script_data.map(
                    |script_data_length| script_data_length.0 .0.len() as i64,
                ),
                storage_slots_count: Some(
                    transaction.storage_slots.len() as i64
                ),
                proof_set_count: Some(transaction.proof_set.len() as i64),
                witnesses_count: Some(transaction.witnesses.len() as i64),
                inputs_count: Some(transaction.inputs.len() as i64),
                outputs_count: Some(transaction.outputs.len() as i64),
                // -- from transactions_data
                transaction_data: transaction.raw_payload.0 .0,
                // -- from transaction_storage_slots (TODO)
                transaction_storage_slots_key: vec![],
                transaction_storage_slots_value: vec![],
                // -- from transaction_witnesses (TODO)
                witness_data: vec![],
                witness_data_length: vec![],
                // -- from transaction_proof_set (TODO)
                transaction_proof_set_proof_hash: vec![],
                // -- from transaction_policies (TODO)
                transaction_policy_type: vec![],
                transaction_policy_data: vec![],
                // -- from transaction_accounts (TODO)
                transaction_account_address: vec![],
                transaction_account_type: vec![],
                // -- timestamps
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
        // Order by block height first
        self.block_height
            .cmp(&other.block_height)
            // Then by transaction index within the block
            .then(self.tx_index.cmp(&other.tx_index))
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
