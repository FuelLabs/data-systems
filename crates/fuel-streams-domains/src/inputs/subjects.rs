use fuel_streams_macros::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use crate::blocks::types::*;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "inputs_coin")]
#[subject(wildcard = "inputs.>")]
#[subject(
    format = "inputs.coin.{block_height}.{tx_id}.{tx_index}.{input_index}.{owner}.{asset}"
)]
pub struct InputsCoinSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<u32>,
    #[subject(sql_column = "owner_id")]
    pub owner: Option<Address>,
    #[subject(sql_column = "asset_id")]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "inputs_contract")]
#[subject(wildcard = "inputs.>")]
#[subject(
    format = "inputs.contract.{block_height}.{tx_id}.{tx_index}.{input_index}.{contract}"
)]
pub struct InputsContractSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<u32>,
    #[subject(sql_column = "contract_id")]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "inputs_message")]
#[subject(wildcard = "inputs.>")]
#[subject(
    format = "inputs.message.{block_height}.{tx_id}.{tx_index}.{input_index}.{sender}.{recipient}"
)]
pub struct InputsMessageSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<u32>,
    #[subject(sql_column = "sender_address")]
    pub sender: Option<Address>,
    #[subject(sql_column = "recipient_address")]
    pub recipient: Option<Address>,
}
