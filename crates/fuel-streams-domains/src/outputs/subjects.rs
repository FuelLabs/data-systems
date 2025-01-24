use fuel_streams_macros::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::OutputType;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_coin")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.coin.>")]
#[subject(custom_where = "output_type = 'coin'")]
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
#[subject(entity = "Output")]
#[subject(query_all = "outputs.contract.>")]
#[subject(custom_where = "output_type = 'contract'")]
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
#[subject(entity = "Output")]
#[subject(query_all = "outputs.change.>")]
#[subject(custom_where = "output_type = 'change'")]
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
#[subject(entity = "Output")]
#[subject(query_all = "outputs.variable.>")]
#[subject(custom_where = "output_type = 'variable'")]
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
#[subject(entity = "Output")]
#[subject(query_all = "outputs.contract_created.>")]
#[subject(custom_where = "output_type = 'contract_created'")]
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

// This subject is used just for query purpose, not for inserting as key
#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.>")]
#[subject(
    format = "outputs.{output_type}.{block_height}.{tx_id}.{tx_index}.{output_index}"
)]
pub struct OutputsSubject {
    pub output_type: Option<OutputType>,
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<u32>,
}
