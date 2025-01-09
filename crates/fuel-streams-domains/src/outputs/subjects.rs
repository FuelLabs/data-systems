use fuel_streams_macros::subject::*;
use fuel_streams_types::*;

use crate::blocks::types::*;

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.coin.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset_id}"]
pub struct OutputsCoinSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<u32>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.contract.{block_height}.{tx_id}.{tx_index}.{output_index}.{contract_id}"]
pub struct OutputsContractSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<u32>,
    pub contract_id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.change.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset_id}"]
pub struct OutputsChangeSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<u32>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.variable.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset_id}"]
pub struct OutputsVariableSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<u32>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.contract_created.{block_height}.{tx_id}.{tx_index}.{output_index}.{contract_id}"]
pub struct OutputsContractCreatedSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<u32>,
    pub contract_id: Option<ContractId>,
}
