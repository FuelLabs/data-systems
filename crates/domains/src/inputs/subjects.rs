use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::{InputType, InputsQuery};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "inputs_coin")]
#[subject(entity = "Input")]
#[subject(query_all = "inputs.coin.>")]
#[subject(custom_where = "input_type = 'coin'")]
#[subject(
    format = "inputs.coin.{block_height}.{tx_id}.{tx_index}.{input_index}.{owner}.{asset}"
)]
pub struct InputsCoinSubject {
    #[subject(
        description = "The height of the block containing this coin input"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this coin input (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(description = "The index of this input within the transaction")]
    pub input_index: Option<i32>,
    #[subject(
        sql_column = "owner_id",
        description = "The address of the coin owner (32 byte string prefixed by 0x)"
    )]
    pub owner: Option<Address>,
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID of the coin (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "inputs_contract")]
#[subject(entity = "Input")]
#[subject(query_all = "inputs.contract.>")]
#[subject(custom_where = "input_type = 'contract'")]
#[subject(
    format = "inputs.contract.{block_height}.{tx_id}.{tx_index}.{input_index}.{contract}"
)]
pub struct InputsContractSubject {
    #[subject(
        description = "The height of the block containing this contract input"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this contract input (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(description = "The index of this input within the transaction")]
    pub input_index: Option<i32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract being called (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "inputs_message")]
#[subject(entity = "Input")]
#[subject(query_all = "inputs.message.>")]
#[subject(custom_where = "input_type = 'message'")]
#[subject(
    format = "inputs.message.{block_height}.{tx_id}.{tx_index}.{input_index}.{sender}.{recipient}"
)]
pub struct InputsMessageSubject {
    #[subject(
        description = "The height of the block containing this message input"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this message input (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(description = "The index of this input within the transaction")]
    pub input_index: Option<i32>,
    #[subject(
        sql_column = "sender_address",
        description = "The address that sent the message (32 byte string prefixed by 0x)"
    )]
    pub sender: Option<Address>,
    #[subject(
        sql_column = "recipient_address",
        description = "The address that will receive the message (32 byte string prefixed by 0x)"
    )]
    pub recipient: Option<Address>,
}

// This subject is used just for query purpose, not for inserting as key
#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "inputs")]
#[subject(entity = "Input")]
#[subject(query_all = "inputs.>")]
#[subject(
    format = "inputs.{input_type}.{block_height}.{tx_id}.{tx_index}.{input_index}"
)]
pub struct InputsSubject {
    #[subject(description = "The type of input (coin, contract, or message)")]
    pub input_type: Option<InputType>,
    #[subject(description = "The height of the block containing this input")]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this input (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(description = "The index of this input within the transaction")]
    pub input_index: Option<i32>,
}

impl From<InputsCoinSubject> for InputsQuery {
    fn from(subject: InputsCoinSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            input_index: subject.input_index,
            input_type: Some(InputType::Coin),
            owner_id: subject.owner.clone(),
            asset_id: subject.asset.clone(),
            ..Default::default()
        }
    }
}

impl From<InputsContractSubject> for InputsQuery {
    fn from(subject: InputsContractSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            input_index: subject.input_index,
            input_type: Some(InputType::Contract),
            contract_id: subject.contract.clone(),
            ..Default::default()
        }
    }
}

impl From<InputsMessageSubject> for InputsQuery {
    fn from(subject: InputsMessageSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            input_index: subject.input_index,
            input_type: Some(InputType::Message),
            sender_address: subject.sender.clone(),
            recipient_address: subject.recipient.clone(),
            ..Default::default()
        }
    }
}

impl From<InputsSubject> for InputsQuery {
    fn from(subject: InputsSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            input_index: subject.input_index,
            input_type: subject.input_type.clone(),
            ..Default::default()
        }
    }
}
