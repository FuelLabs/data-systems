use fuel_streams_macros::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use crate::blocks::types::*;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_coin")]
#[subject(wildcard = "outputs.>")]
#[subject(
    format = "outputs.coin.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset}"
)]
pub struct OutputsCoinSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<u32>,
    #[subject(sql_column = "to_address")]
    pub to: Option<Address>,
    #[subject(sql_column = "asset_id")]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_contract")]
#[subject(wildcard = "outputs.>")]
#[subject(
    format = "outputs.contract.{block_height}.{tx_id}.{tx_index}.{output_index}.{contract}"
)]
pub struct OutputsContractSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<u32>,
    #[subject(sql_column = "contract_id")]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_change")]
#[subject(wildcard = "outputs.>")]
#[subject(
    format = "outputs.change.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset}"
)]
pub struct OutputsChangeSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<u32>,
    #[subject(sql_column = "to_address")]
    pub to: Option<Address>,
    #[subject(sql_column = "asset_id")]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_variable")]
#[subject(wildcard = "outputs.>")]
#[subject(
    format = "outputs.variable.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset}"
)]
pub struct OutputsVariableSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<u32>,
    #[subject(sql_column = "to_address")]
    pub to: Option<Address>,
    #[subject(sql_column = "asset_id")]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_contract_created")]
#[subject(wildcard = "outputs.>")]
#[subject(
    format = "outputs.contract_created.{block_height}.{tx_id}.{tx_index}.{output_index}.{contract}"
)]
pub struct OutputsContractCreatedSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<u32>,
    #[subject(sql_column = "contract_id")]
    pub contract: Option<ContractId>,
}
