use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::OutputsQuery;
use crate::infra::QueryPagination;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_coin")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.coin.>")]
#[subject(custom_where = "output_type = 'coin'")]
#[subject(
    format = "outputs.coin.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset}"
)]
pub struct OutputsCoinSubject {
    #[subject(
        description = "The height of the block containing this coin output"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this coin output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<i32>,
    #[subject(
        sql_column = "to_address",
        description = "The recipient address of the coin output (32 byte string prefixed by 0x)"
    )]
    pub to: Option<Address>,
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID of the coin (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_contract")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.contract.>")]
#[subject(custom_where = "output_type = 'contract'")]
#[subject(
    format = "outputs.contract.{block_height}.{tx_id}.{tx_index}.{output_index}.{contract}"
)]
pub struct OutputsContractSubject {
    #[subject(
        description = "The height of the block containing this contract output"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this contract output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<i32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_change")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.change.>")]
#[subject(custom_where = "output_type = 'change'")]
#[subject(
    format = "outputs.change.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset}"
)]
pub struct OutputsChangeSubject {
    #[subject(
        description = "The height of the block containing this change output"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this change output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<i32>,
    #[subject(
        sql_column = "to_address",
        description = "The recipient address of the change output (32 byte string prefixed by 0x)"
    )]
    pub to: Option<Address>,
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID of the change output (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_variable")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.variable.>")]
#[subject(custom_where = "output_type = 'variable'")]
#[subject(
    format = "outputs.variable.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset}"
)]
pub struct OutputsVariableSubject {
    #[subject(
        description = "The height of the block containing this variable output"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this variable output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<i32>,
    #[subject(
        sql_column = "to_address",
        description = "The recipient address of the variable output (32 byte string prefixed by 0x)"
    )]
    pub to: Option<Address>,
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID of the variable output (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_contract_created")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.contract_created.>")]
#[subject(custom_where = "output_type = 'contract_created'")]
#[subject(
    format = "outputs.contract_created.{block_height}.{tx_id}.{tx_index}.{output_index}.{contract}"
)]
pub struct OutputsContractCreatedSubject {
    #[subject(
        description = "The height of the block containing this contract creation output"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this contract creation output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<i32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the created contract (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
}

// This subject is used just for query purpose, not for inserting as key
#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.>")]
#[subject(
    format = "outputs.{output_type}.{block_height}.{tx_id}.{tx_index}.{output_index}"
)]
pub struct OutputsSubject {
    #[subject(
        description = "The type of output (coin, contract, change, variable, or contract_created)"
    )]
    pub output_type: Option<OutputType>,
    #[subject(description = "The height of the block containing this output")]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<i32>,
}

impl From<OutputsCoinSubject> for OutputsQuery {
    fn from(subject: OutputsCoinSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            output_index: subject.output_index,
            output_type: Some(OutputType::Coin),
            to_address: subject.to.clone(),
            asset_id: subject.asset.clone(),
            pagination: QueryPagination::default(),
            ..Default::default()
        }
    }
}

impl From<OutputsContractSubject> for OutputsQuery {
    fn from(subject: OutputsContractSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            output_index: subject.output_index,
            output_type: Some(OutputType::Contract),
            contract_id: subject.contract.clone(),
            pagination: QueryPagination::default(),
            ..Default::default()
        }
    }
}

impl From<OutputsChangeSubject> for OutputsQuery {
    fn from(subject: OutputsChangeSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            output_index: subject.output_index,
            output_type: Some(OutputType::Change),
            to_address: subject.to.clone(),
            asset_id: subject.asset.clone(),
            pagination: QueryPagination::default(),
            ..Default::default()
        }
    }
}

impl From<OutputsVariableSubject> for OutputsQuery {
    fn from(subject: OutputsVariableSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            output_index: subject.output_index,
            output_type: Some(OutputType::Variable),
            to_address: subject.to.clone(),
            asset_id: subject.asset.clone(),
            pagination: QueryPagination::default(),
            ..Default::default()
        }
    }
}

impl From<OutputsContractCreatedSubject> for OutputsQuery {
    fn from(subject: OutputsContractCreatedSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            output_index: subject.output_index,
            output_type: Some(OutputType::ContractCreated),
            contract_id: subject.contract.clone(),
            pagination: QueryPagination::default(),
            ..Default::default()
        }
    }
}

impl From<OutputsSubject> for OutputsQuery {
    fn from(subject: OutputsSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            output_index: subject.output_index,
            output_type: subject.output_type,
            ..Default::default()
        }
    }
}
