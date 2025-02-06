use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::types::*;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "utxos")]
#[subject(entity = "Utxo")]
#[subject(query_all = "utxos.>")]
#[subject(
    format = "utxos.{block_height}.{tx_id}.{tx_index}.{input_index}.{utxo_type}.{utxo_id}"
)]
pub struct UtxosSubject {
    #[subject(description = "The height of the block containing this UTXO")]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this UTXO (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(description = "The index of the input within the transaction")]
    pub input_index: Option<u32>,
    #[subject(description = "The type of UTXO (coin, message, or contract)")]
    pub utxo_type: Option<UtxoType>,
    #[subject(
        description = "The unique identifier for this UTXO (32 byte string prefixed by 0x)"
    )]
    pub utxo_id: Option<HexData>,
}
