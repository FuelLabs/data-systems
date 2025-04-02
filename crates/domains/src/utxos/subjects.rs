use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::{types::*, UtxosQuery};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "utxos")]
#[subject(entity = "Utxo")]
#[subject(query_all = "utxos.>")]
#[subject(
    format = "utxos.{block_height}.{tx_id}.{tx_index}.{input_index}.{utxo_type}.{utxo_id}.{contract_id}"
)]
pub struct UtxosSubject {
    #[subject(description = "The height of the block containing this UTXO")]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this UTXO (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(description = "The index of the input within the transaction")]
    pub input_index: Option<i32>,
    #[subject(description = "The type of UTXO (coin, message, or contract)")]
    pub utxo_type: Option<UtxoType>,
    #[subject(
        description = "The unique identifier for this UTXO (32 byte string prefixed by 0x)"
    )]
    pub utxo_id: Option<HexData>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract that returned (32 byte string prefixed by 0x)"
    )]
    pub contract_id: Option<ContractId>,
}

impl From<UtxosSubject> for UtxosQuery {
    fn from(subject: UtxosSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            input_index: subject.input_index,
            utxo_type: subject.utxo_type.as_ref().map(|t| match t {
                UtxoType::Coin => InputType::Coin,
                UtxoType::Contract => InputType::Contract,
                UtxoType::Message => InputType::Message,
            }),
            utxo_id: subject.utxo_id.clone(),
            contract_id: subject.contract_id.clone(),
            ..Default::default()
        }
    }
}
