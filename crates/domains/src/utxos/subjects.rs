use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::{Utxo, UtxosQuery};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "utxos")]
#[subject(entity = "Utxo")]
#[subject(query_all = "utxos.>")]
#[subject(
    format = "utxos.{block_height}.{tx_id}.{tx_index}.{input_index}.{output_index}.{status}.{utxo_type}.{asset_id}.{utxo_id}.{from}.{to}.{contract_id}"
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
    #[subject(description = "The index of the output within the transaction")]
    pub output_index: Option<i32>,
    #[subject(description = "The type of UTXO (coin, message, or contract)")]
    pub utxo_type: Option<UtxoType>,
    #[subject(description = "The ID of the asset associated with this UTXO")]
    pub asset_id: Option<AssetId>,
    #[subject(
        description = "The unique identifier for this UTXO (32 byte string prefixed by 0x)"
    )]
    pub utxo_id: Option<UtxoId>,
    #[subject(
        sql_column = "from_address",
        description = "The address of the sender (32 byte string prefixed by 0x)"
    )]
    pub from: Option<Address>,
    #[subject(
        sql_column = "to_address",
        description = "The address of the recipient (32 byte string prefixed by 0x)"
    )]
    pub to: Option<Address>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract that returned (32 byte string prefixed by 0x)"
    )]
    pub contract_id: Option<ContractId>,
    #[subject(description = "The status of the UTXO (unspent or spent)")]
    pub status: Option<UtxoStatus>,
}

impl From<Utxo> for UtxosSubject {
    fn from(utxo: Utxo) -> Self {
        Self {
            status: Some(utxo.status),
            tx_id: Some(utxo.tx_id),
            utxo_type: Some(utxo.r#type),
            asset_id: utxo.asset_id,
            utxo_id: Some(utxo.utxo_id),
            from: utxo.from,
            to: utxo.to,
            contract_id: utxo.contract_id,
            ..Default::default()
        }
    }
}

impl From<UtxosSubject> for UtxosQuery {
    fn from(subject: UtxosSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            input_index: subject.input_index,
            output_index: subject.output_index,
            r#type: subject.utxo_type,
            utxo_id: subject.utxo_id.clone(),
            contract_id: subject.contract_id.clone(),
            status: subject.status,
            from: subject.from.clone(),
            to: subject.to.clone(),
            asset_id: subject.asset_id.clone(),
            ..Default::default()
        }
    }
}
