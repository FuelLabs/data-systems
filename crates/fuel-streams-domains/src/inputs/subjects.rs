use fuel_streams_macros::subject::*;
use fuel_streams_types::*;

use crate::blocks::types::*;

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "inputs.>"]
#[subject_format = "inputs.coin.{block_height}.{tx_id}.{tx_index}.{input_index}.{owner}.{asset_id}"]
pub struct InputsCoinSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<u32>,
    pub owner: Option<Address>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "inputs.>"]
#[subject_format = "inputs.contract.{block_height}.{tx_id}.{tx_index}.{input_index}.{contract_id}"]
pub struct InputsContractSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<u32>,
    pub contract_id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "inputs.>"]
#[subject_format = "inputs.message.{block_height}.{tx_id}.{tx_index}.{input_index}.{sender}.{recipient}"]
pub struct InputsMessageSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<u32>,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
}
